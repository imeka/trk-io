use std::{fs::File, io::BufReader, path::Path};

use anyhow::{Context, Result};
use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt};
use nalgebra::Vector3;

use crate::{
    cheader::Endianness,
    tractogram::{Point, Points, Streamlines, Tractogram, TractogramItem},
    Affine, ArraySequence, Header, Spacing, Translation,
};

pub struct Reader {
    reader: BufReader<File>,
    endianness: Endianness,
    pub header: Header,

    raw: bool,
    voxel_space: bool,

    floats_per_point: usize,
    buffer: Vec<f32>,
}

impl Reader {
    /// Create an object to read all points of a TrackVis file in world space.
    ///
    /// Will also read the scalars and properties, if requested.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Reader> {
        let f = File::open(path.as_ref())
            .with_context(|| format!("Failed to load {:?}", path.as_ref()))?;
        let mut reader = BufReader::new(f);
        let (header, endianness) = Header::read(&mut reader)?;
        let floats_per_point = 3 + header.scalars_name.len();
        let buffer = Vec::with_capacity(300);

        let raw = false;
        let voxel_space = false;

        Ok(Reader { reader, endianness, header, raw, voxel_space, floats_per_point, buffer })
    }

    /// Modify the affine in order to read all streamlines in voxel space.
    ///
    /// Once this function is called, it's not possible to revert to reading in world space.
    ///
    /// Panics if `raw` has been called.
    pub fn to_voxel_space(mut self, spacing: Spacing) -> Self {
        if self.raw {
            panic!("Can't use raw + voxel space reading");
        }

        self.voxel_space = true;
        self.header.affine_to_rasmm =
            Affine::from_diagonal(&Vector3::new(1.0 / spacing.x, 1.0 / spacing.y, 1.0 / spacing.z));
        self.header.translation = Translation::zeros();
        self
    }

    /// Read the points as they are written on disk, without any transformation.
    ///
    /// Panics if `to_voxel_space` has been called.
    pub fn raw(mut self) -> Self {
        if self.voxel_space {
            panic!("Can't use voxel space + raw reading");
        }

        self.raw = true;
        self
    }

    /// Iterate only on streamlines (`Vec<Point>`), ignoring scalars and properties.
    pub fn into_streamlines_iter(self) -> StreamlinesIter {
        StreamlinesIter { reader: self }
    }

    /// Read the complete tractogram, that is, all points, scalars and properties, if any.
    pub fn tractogram(&mut self) -> Tractogram {
        match self.endianness {
            Endianness::Little => self.read_tractogram_::<LittleEndian>(),
            Endianness::Big => self.read_tractogram_::<BigEndian>(),
        }
    }

    /// Read all points, ignoring the scalars and properties.
    pub fn streamlines(&mut self) -> Streamlines {
        match self.endianness {
            Endianness::Little => self.read_points_::<LittleEndian>(),
            Endianness::Big => self.read_points_::<BigEndian>(),
        }
    }

    fn read_tractogram_<E: ByteOrder>(&mut self) -> Tractogram {
        // TODO Anything we can do to reserve?
        let mut lengths = Vec::new();
        let mut v = Vec::with_capacity(300);
        let mut scalars = ArraySequence::with_capacity(300);
        let mut properties = ArraySequence::with_capacity(300);
        while let Ok(nb_points) = self.reader.read_i32::<E>() {
            lengths.push(nb_points as usize);
            self.read_streamline::<E>(&mut v, &mut scalars, nb_points as usize);
            self.read_properties_to_arr::<E>(&mut properties);
        }

        self.buffer = vec![];
        Tractogram::new(Streamlines::new(lengths, v), scalars, properties)
    }

    fn read_points_<E: ByteOrder>(&mut self) -> Streamlines {
        // TODO Anything we can do to reserve?
        let mut lengths = Vec::new();
        let mut v = Vec::with_capacity(300);
        while let Ok(nb_points) = self.reader.read_i32::<E>() {
            lengths.push(nb_points as usize);
            self.read_streamline_fast::<E>(&mut v, nb_points as usize);
        }

        self.buffer = vec![];
        Streamlines::new(lengths, v)
    }

    fn read_streamline<E: ByteOrder>(
        &mut self,
        points: &mut Points,
        scalars: &mut ArraySequence<f32>,
        nb_points: usize,
    ) {
        self.read_floats::<E>(nb_points);
        for floats in self.buffer.chunks(self.floats_per_point) {
            self.add_points(points, floats);
            for f in &floats[3..] {
                scalars.push(*f);
            }
        }
        scalars.end_push();
    }

    /// Ignore the scalars and properties.
    fn read_streamline_fast<E: ByteOrder>(&mut self, points: &mut Points, nb_points: usize) {
        self.read_floats::<E>(nb_points);
        for floats in self.buffer.chunks(self.floats_per_point) {
            self.add_points(points, floats);
            // Scalars have been read in `floats`, but we do not save them
        }

        // Properties must be read to advance the cursor, but we do not save them
        for _ in 0..self.header.properties_name.len() {
            self.reader.read_f32::<E>().unwrap();
        }
    }

    /// Read all points and scalars for the current streamline.
    ///
    /// Simply chunk the result by `nb_floats_per_point` to get the 3D point and the scalars.
    fn read_floats<E: ByteOrder>(&mut self, nb_points: usize) {
        // Vec::resize never decreases capacity, it can only increase it so there won't be any
        // useless allocation.
        let nb_floats = nb_points * self.floats_per_point;
        self.buffer.resize(nb_floats, 0.0);
        self.reader.read_f32_into::<E>(self.buffer.as_mut_slice()).unwrap();
    }

    #[inline(always)]
    fn add_points(&self, points: &mut Points, floats: &[f32]) {
        let mut p = Point::new(floats[0], floats[1], floats[2]);
        if !self.raw {
            p = (self.header.affine_to_rasmm * p) + self.header.translation;
        }
        points.push(p);
    }

    fn read_properties_to_arr<E: ByteOrder>(&mut self, properties: &mut ArraySequence<f32>) {
        for _ in 0..self.header.properties_name.len() {
            properties.push(self.reader.read_f32::<E>().unwrap());
        }
        properties.end_push();
    }

    fn read_properties_to_vec<E: ByteOrder>(&mut self, properties: &mut Vec<f32>) {
        for _ in 0..self.header.properties_name.len() {
            properties.push(self.reader.read_f32::<E>().unwrap());
        }
    }

    fn read_nb_points(&mut self) -> Option<usize> {
        match self.endianness {
            Endianness::Little => self.reader.read_i32::<LittleEndian>(),
            Endianness::Big => self.reader.read_i32::<BigEndian>(),
        }
        .ok()
        .map(|nb_points| nb_points as usize)
    }
}

impl Iterator for Reader {
    type Item = TractogramItem;

    fn next(&mut self) -> Option<TractogramItem> {
        let nb_points = self.read_nb_points()?;
        let mut streamline = Vec::with_capacity(nb_points);
        let mut scalars = ArraySequence::with_capacity(nb_points * self.header.scalars_name.len());
        let mut properties = Vec::with_capacity(self.header.properties_name.len());
        match self.endianness {
            Endianness::Little => {
                self.read_streamline::<LittleEndian>(&mut streamline, &mut scalars, nb_points);
                self.read_properties_to_vec::<LittleEndian>(&mut properties);
            }
            Endianness::Big => {
                self.read_streamline::<BigEndian>(&mut streamline, &mut scalars, nb_points);
                self.read_properties_to_vec::<BigEndian>(&mut properties);
            }
        };
        Some((streamline, scalars, properties))
    }
}

pub struct StreamlinesIter {
    reader: Reader,
}

impl Iterator for StreamlinesIter {
    type Item = Points;

    fn next(&mut self) -> Option<Points> {
        let nb_points = self.reader.read_nb_points()?;
        let mut streamline = Vec::with_capacity(nb_points);
        match self.reader.endianness {
            Endianness::Little => {
                self.reader.read_streamline_fast::<LittleEndian>(&mut streamline, nb_points);
            }
            Endianness::Big => {
                self.reader.read_streamline_fast::<BigEndian>(&mut streamline, nb_points);
            }
        };
        Some(streamline)
    }
}
