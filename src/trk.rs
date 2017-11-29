
use std::fs::{File};
use std::io::{BufReader, BufWriter, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use {Affine, Translation};
use header::{Header, HEADER_SIZE, read_header};
use streamlines::{Streamlines, Point, Points};

pub struct Reader {
    reader: BufReader<File>,
    header: Header,
    affine: Affine,
    translation: Translation,

    nb_floats_per_point: usize,
    float_buffer: Vec<f32>
}

impl Reader {
    pub fn new(path: &str) -> Reader {
        let header = read_header(path);
        let (affine, translation) = header.get_affine();
        let nb_floats_per_point = 3 + header.n_scalars as usize;

        let mut f = File::open(path).expect("Can't read trk file.");
        f.seek(SeekFrom::Start(HEADER_SIZE as u64)).unwrap();

        Reader {
            reader: BufReader::new(f),
            header, affine, translation, nb_floats_per_point,
            float_buffer: Vec::with_capacity(300)
        }
    }

    pub fn read_all(&mut self) -> Streamlines {
        let mut v = Vec::with_capacity(3000);
        let mut lengths = Vec::new();
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
        for _ in 0..self.header.n_properties {
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

pub fn write_streamlines(
    header: &Header,
    streamlines: &Streamlines,
    path: &str)
{
    let (affine, translation) = header.get_affine();
    let affine = affine.try_inverse().unwrap();

    let f = File::create(path).expect("Can't create new trk file.");
    let mut writer = BufWriter::new(f);

    let header = Header {
        n_count: streamlines.lengths.len() as i32,
        ..Header::default()
    };
    header.write(&mut writer);

    for streamline in streamlines {
        writer.write_i32::<LittleEndian>(streamline.len() as i32).unwrap();
        for p in streamline {
            let p = (p - translation) * affine;
            writer.write_f32::<LittleEndian>(p.x).unwrap();
            writer.write_f32::<LittleEndian>(p.y).unwrap();
            writer.write_f32::<LittleEndian>(p.z).unwrap();
        }
    }
}
