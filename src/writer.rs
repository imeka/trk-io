use std::{fs::File, io::BufWriter, path::Path};

use anyhow::Result;
use byteorder::WriteBytesExt;
use nalgebra::Vector4;

use crate::{
    affine::get_affine_and_translation,
    tractogram::{Point, RefTractogramItem, Tractogram, TractogramItem},
    Affine, Affine4, CHeader, Header, Spacing, Translation, TrkEndianness,
};

macro_rules! write_streamline {
    ($writer:ident, $streamline:expr, $scalars:expr, $properties:expr) => {
        if $writer.nb_scalars == 0 {
            $streamline.write($writer);
        } else {
            $writer.writer.write_i32::<TrkEndianness>($streamline.len() as i32).unwrap();
            $writer.real_n_count += 1;

            let scalars = $scalars.chunks($writer.nb_scalars);
            for (p, scalars) in $streamline.into_iter().zip(scalars) {
                $writer.write_point(&p);
                $writer.write_f32s(scalars);
            }
        }

        $writer.write_f32s($properties);
    };
    // Fast method, without scalars and properties
    ($writer:ident, $streamline:expr, $nb_points:expr) => {
        $writer.writer.write_i32::<TrkEndianness>($nb_points as i32).unwrap();
        for p in $streamline {
            $writer.write_point(&p);
        }
        $writer.real_n_count += 1;
    };
}

pub struct Writer {
    writer: BufWriter<File>,
    pub affine4: Affine4,
    affine: Affine,
    translation: Translation,
    nb_scalars: usize,

    real_n_count: i32,
}

pub trait Writable {
    fn write(self, w: &mut Writer);
}

impl Writable for Tractogram {
    fn write(self, w: &mut Writer) {
        for item in &self {
            item.write(w);
        }
    }
}

impl Writable for TractogramItem {
    fn write(self, writer: &mut Writer) {
        let (streamline, scalars, properties) = self;
        write_streamline!(writer, streamline, scalars.data.as_slice(), &properties);
    }
}

impl<'data> Writable for RefTractogramItem<'data> {
    fn write(self, writer: &mut Writer) {
        let (streamline, scalars, properties) = self;
        write_streamline!(writer, streamline, scalars, properties);
    }
}

impl<'data> Writable for &'data [Point] {
    fn write(self, writer: &mut Writer) {
        write_streamline!(writer, self, self.len());
    }
}

impl Writer {
    pub fn new<P: AsRef<Path>>(path: P, reference: Option<Header>) -> Result<Writer> {
        let f = File::create(path).expect("Can't create new trk file.");
        let mut writer = BufWriter::new(f);

        let (header, affine4) = match reference {
            Some(header) => {
                // We are only interested in the inversed affine
                let affine4 = header
                    .affine4_to_rasmm
                    .try_inverse()
                    .expect("Unable to inverse 4x4 affine matrix");
                (header, affine4)
            }
            None => (Header::default(), Affine4::identity()),
        };
        header.write(&mut writer)?;
        let (affine, translation) = get_affine_and_translation(&affine4);
        let nb_scalars = header.scalars_name.len();

        let real_n_count = 0;

        Ok(Writer { writer, affine4, affine, translation, real_n_count, nb_scalars })
    }

    /// Modifies the affine in order to write all streamlines from voxel space to the right
    /// coordinate space on disk.
    ///
    /// The resulting file will only valid if the streamlines were read using `to_voxel_space`.
    pub fn from_voxel_space(mut self, spacing: Spacing) -> Self {
        self.affine = Affine::from_diagonal(&spacing);
        self.affine4 = Affine4::from_diagonal(&Vector4::new(spacing.x, spacing.y, spacing.z, 1.0));
        self.translation = Translation::zeros();
        self
    }

    /// Resets the affine so that no transformation is applied to the points.
    ///
    /// The TrackVis header (on disk) will NOT be modified.
    pub fn reset_affine(&mut self) {
        self.affine4 = Affine4::identity();
        self.affine = Affine::identity();
        self.translation = Translation::zeros();
    }

    /// Applies a new affine over the current affine.
    ///
    /// The TrackVis header (on disk) will **not** be modified.
    pub fn apply_affine(&mut self, affine: &Affine4) {
        self.affine4 = self.affine4 * affine;
        let (affine, translation) = get_affine_and_translation(&self.affine4);
        self.affine = affine;
        self.translation = translation;
    }

    pub fn write<T: Writable>(&mut self, data: T) {
        data.write(self);
    }

    pub fn write_from_iter<I>(&mut self, streamline: I, len: usize)
    where
        I: IntoIterator<Item = Point>,
    {
        write_streamline!(self, streamline, len);
    }

    fn write_point(&mut self, p: &Point) {
        let p = self.affine * p + self.translation;
        self.writer.write_f32::<TrkEndianness>(p.x).unwrap();
        self.writer.write_f32::<TrkEndianness>(p.y).unwrap();
        self.writer.write_f32::<TrkEndianness>(p.z).unwrap();
    }

    fn write_f32s(&mut self, data: &[f32]) {
        for &d in data {
            self.writer.write_f32::<TrkEndianness>(d).unwrap();
        }
    }
}

// Finally write `n_count`
impl Drop for Writer {
    fn drop(&mut self) {
        CHeader::seek_n_count_field(&mut self.writer)
            .expect("Unable to seek to 'n_count' field before closing trk file.");
        self.writer
            .write_i32::<TrkEndianness>(self.real_n_count)
            .expect("Unable to write 'n_count' field before closing trk file.");
    }
}
