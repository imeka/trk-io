
use std::fs::File;
use std::io::BufWriter;

use byteorder::{LittleEndian, WriteBytesExt};

use {Affine, Affine4, CHeader, Header, Point, Streamlines, Translation};
use affine::get_affine_and_translation;

pub struct Writer {
    writer: BufWriter<File>,
    pub affine4: Affine4,
    affine: Affine,
    translation: Translation,
    real_n_count: i32
}

impl Writer {
    pub fn new(path: &str, reference: Option<Header>) -> Writer {
        let f = File::create(path).expect("Can't create new trk file.");
        let mut writer = BufWriter::with_capacity(1024, f);

        let header = match reference {
            Some(r) => r.clone(),
            None => Header::default()
        };
        header.write(&mut writer);

        // We are only interested in the inversed affine
        let affine4 = header.affine4.try_inverse().unwrap();
        let (affine, translation) = get_affine_and_translation(&affine4);

        Writer { writer, affine4, affine, translation, real_n_count: 0 }
    }

    pub fn transform_to_world(&mut self, to_world: &Affine4) {
        self.affine4 = to_world * self.affine4;
        let (affine, translation) = get_affine_and_translation(&self.affine4);
        self.affine = affine;
        self.translation = translation;
    }

    pub fn write_all(&mut self, streamlines: &Streamlines) {
        for streamline in streamlines {
            self.write(streamline);
        }
    }

    pub fn write(&mut self, streamline: &[Point]) {
        self.writer.write_i32::<LittleEndian>(
            streamline.len() as i32).unwrap();
        for p in streamline {
            self.write_point(*p);
        }
        self.real_n_count += 1;
    }

    pub fn write_from_iter<I>(&mut self, streamline: I, len: usize)
        where I: IntoIterator<Item = Point>
    {
        self.writer.write_i32::<LittleEndian>(len as i32).unwrap();
        for p in streamline {
            self.write_point(p);
        }
        self.real_n_count += 1;
    }

    fn write_point(&mut self, p: Point) {
        let p = p * self.affine + self.translation;
        self.writer.write_f32::<LittleEndian>(p.x).unwrap();
        self.writer.write_f32::<LittleEndian>(p.y).unwrap();
        self.writer.write_f32::<LittleEndian>(p.z).unwrap();
    }
}

// Finally write `n_count`
impl Drop for Writer {
    fn drop(&mut self) {
        CHeader::seek_n_count_field(&mut self.writer);
        self.writer.write_i32::<LittleEndian>(self.real_n_count).unwrap();
    }
}
