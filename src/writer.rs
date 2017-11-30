
use std::fs::File;
use std::io::BufWriter;

use byteorder::{LittleEndian, WriteBytesExt};

use {Affine, CHeader, Header, Point, Streamlines, Translation};

pub struct Writer {
    writer: BufWriter<File>,
    affine: Affine,
    translation: Translation,
    real_n_count: i32
}

impl Writer {
    pub fn new(path: &str, reference: Option<Header>) -> Writer
    {
        let f = File::create(path).expect("Can't create new trk file.");
        let mut writer = BufWriter::new(f);

        let header = match reference {
            Some(r) => r.clone(),
            None => Header::default()
        };
        header.write(&mut writer);

        Writer {
            writer,
            affine: header.affine.try_inverse().unwrap(),
            translation: header.translation,
            real_n_count: 0
        }
    }

    pub fn write_all(&mut self, streamlines: &Streamlines) {
        for streamline in streamlines {
            self.write(streamline);
        }
    }

    pub fn write(&mut self, streamline: &[Point]) {
        self.writer.write_i32::<LittleEndian>(streamline.len() as i32).unwrap();
        for p in streamline {
            let p = (p - self.translation) * self.affine;
            self.writer.write_f32::<LittleEndian>(p.x).unwrap();
            self.writer.write_f32::<LittleEndian>(p.y).unwrap();
            self.writer.write_f32::<LittleEndian>(p.z).unwrap();
        }
        self.real_n_count += 1;
    }
}

// Finally write `n_count`
impl Drop for Writer {
    fn drop(&mut self) {
        CHeader::seek_n_count_field(&mut self.writer);
        self.writer.write_i32::<LittleEndian>(self.real_n_count).unwrap();
    }
}
