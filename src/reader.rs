
use std::fs::{File};
use std::io::BufReader;

use byteorder::{LittleEndian, ReadBytesExt};

use {Affine, CHeader, Header, Point, Points, Streamlines, Translation};

pub struct Reader {
    reader: BufReader<File>,
    pub header: Header,
    pub affine: Affine,
    pub translation: Translation,

    nb_floats_per_point: usize,
    float_buffer: Vec<f32>
}

impl Reader {
    pub fn new(path: &str) -> Reader {
        let header = Header::read(path);
        let affine = header.affine;
        let translation = header.translation;
        let nb_floats_per_point = 3 + header.scalars_name.len() as usize;

        let mut f = File::open(path).expect("Can't read trk file.");
        CHeader::seek_end(&mut f);

        Reader {
            reader: BufReader::new(f),
            header, affine, translation,
            nb_floats_per_point,
            float_buffer: Vec::with_capacity(300)
        }
    }

    pub fn read_all(&mut self) -> Streamlines {
        let mut lengths = Vec::new();
        let mut v = Vec::with_capacity(300);
        loop {
            if let Ok(nb_points) = self.reader.read_i32::<LittleEndian>() {
                lengths.push(nb_points as usize);
                self.read_streamline(&mut v, nb_points as usize);
            }
            else { break; }
        }

        Streamlines::new(lengths, v)
    }

    fn read_streamline(&mut self, points: &mut Points, nb_points: usize) {
        // Vec::resize never decreases capacity, it can only increase it
        // so there won't be any useless allocation.
        let nb_floats = nb_points * self.nb_floats_per_point;
        self.float_buffer.resize(nb_floats as usize, 0.0);
        unsafe {
            self.reader.read_f32_into_unchecked::<LittleEndian>(
                self.float_buffer.as_mut_slice()).unwrap();
        }

        for floats in self.float_buffer.chunks(self.nb_floats_per_point) {
            let p = Point::new(floats[0], floats[1], floats[2]);
            points.push((p * self.affine) + self.translation);
        }

        // Ignore properties for now
        for _ in 0..self.header.properties_name.len() {
            self.reader.read_f32::<LittleEndian>().unwrap();
        }
    }
}

impl Iterator for Reader {
    type Item = Points;

    fn next(&mut self) -> Option<Points> {
        if let Ok(nb_points) = self.reader.read_i32::<LittleEndian>() {
            let mut points = Vec::with_capacity(nb_points as usize);
            self.read_streamline(&mut points, nb_points as usize);
            Some(points)
        } else {
            None
        }
    }
}
