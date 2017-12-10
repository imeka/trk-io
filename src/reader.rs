
use std::fs::{File};
use std::io::BufReader;

use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt};

use {Affine, Header, Point, Points, Streamlines, Translation};
use cheader::{CHeader, Endianness};

pub struct Reader {
    reader: BufReader<File>,
    endianness: Endianness,
    pub header: Header,
    pub affine: Affine,
    pub translation: Translation,

    nb_floats_per_point: usize,
    float_buffer: Vec<f32>
}

impl Reader {
    pub fn new(path: &str) -> Reader {
        let (header, endianness) = Header::read(path);
        let affine = header.affine;
        let translation = header.translation;
        let nb_floats_per_point = 3 + header.scalars.len() as usize;

        let mut f = File::open(path).expect("Can't read trk file.");
        CHeader::seek_end(&mut f);

        Reader {
            reader: BufReader::new(f),
            endianness, header, affine, translation, nb_floats_per_point,
            float_buffer: Vec::with_capacity(300)
        }
    }

    pub fn read_all(&mut self) -> Streamlines {
        match self.endianness {
            Endianness::Little => self.read_all_::<LittleEndian>(),
            Endianness::Big => self.read_all_::<BigEndian>()
        }
    }

    fn read_all_<E: ByteOrder>(&mut self) -> Streamlines {
        self.header.scalars.reserve(300);
        self.header.properties.reserve(self.header.nb_streamlines);

        let mut lengths = Vec::new();
        let mut v = Vec::with_capacity(300);
        while let Ok(nb_points) = self.reader.read_i32::<E>() {
            lengths.push(nb_points as usize);
            self.read_streamline::<E>(&mut v, nb_points as usize);
        }
        self.float_buffer = vec![];
        Streamlines::new(lengths, v)
    }

    fn read_streamline<E: ByteOrder>(
        &mut self,
        points: &mut Points,
        nb_points: usize)
    {
        // Vec::resize never decreases capacity, it can only increase it
        // so there won't be any useless allocation.
        let nb_floats = nb_points * self.nb_floats_per_point;
        self.float_buffer.resize(nb_floats as usize, 0.0);
        self.reader.read_f32_into_unchecked::<E>(
            self.float_buffer.as_mut_slice()).unwrap();

        for floats in self.float_buffer.chunks(self.nb_floats_per_point) {
            let p = Point::new(floats[0], floats[1], floats[2]);
            points.push((p * self.affine) + self.translation);

            for (&mut (_, ref mut scalar), f) in self.header.scalars.iter_mut()
                                                 .zip(&floats[3..]) {
                scalar.push(*f);
            }
        }

        for &mut (_, ref mut scalar) in &mut self.header.scalars {
            scalar.end_push();
        }

        for &mut (_, ref mut property) in &mut self.header.properties {
            property.push(self.reader.read_f32::<E>().unwrap());
        }
    }
}

impl Iterator for Reader {
    type Item = Points;

    fn next(&mut self) -> Option<Points> {
        if let Ok(nb_points) = match self.endianness {
            Endianness::Little => self.reader.read_i32::<LittleEndian>(),
            Endianness::Big => self.reader.read_i32::<BigEndian>()
        } {
            let mut points = Vec::with_capacity(nb_points as usize);
            match self.endianness {
                Endianness::Little => self.read_streamline::<LittleEndian>(
                    &mut points, nb_points as usize),
                Endianness::Big => self.read_streamline::<BigEndian>(
                    &mut points, nb_points as usize)
            };
            Some(points)
        } else {
            None
        }
    }
}
