use std::fs::File;
use std::io::{BufWriter, Result};
use std::path::Path;

use byteorder::{LittleEndian, WriteBytesExt};

use tractogram::{Point, RefTractogramItem, Tractogram, TractogramItem};
use {Affine, Affine4, CHeader, Header, Translation};
use affine::get_affine_and_translation;

pub struct Writer {
    writer: BufWriter<File>,
    pub affine4: Affine4,
    affine: Affine,
    translation: Translation,
    real_n_count: i32
}

pub trait Writable {
    fn write(self, w: &mut Writer);
}

impl Writable for Tractogram {
    fn write(self, w: &mut Writer) {
        // TODO Don't use destructuring, use the tractogram iteration
        let Tractogram { streamlines, scalars, properties } = self;

        // Transform scalars and properties into vector of iterators, so we always know the exact
        // positions where we were.
        let mut scalars = scalars.into_iter().map(|v| v.data.into_iter()).collect::<Vec<_>>();
        let mut properties = properties.into_iter().map(|v| v.into_iter()).collect::<Vec<_>>();

        for streamline in streamlines.into_iter() {
            w.writer.write_i32::<LittleEndian>(streamline.len() as i32).unwrap();
            for p in streamline {
                w.write_point_and_scalars(p, &mut scalars);
            }
            for property in properties.iter_mut() {
                let property = property.next().expect("Missing some properties");
                w.writer.write_f32::<LittleEndian>(property).unwrap();
            }
            w.real_n_count += 1;
        }
    }
}

impl Writable for TractogramItem {
    fn write(self, w: &mut Writer) {
        let (streamline, scalars, properties) = self;

        // Transform scalars into a vector of iterators, so we always know where we were
        let mut scalars = scalars.into_iter().map(|v| v.into_iter()).collect::<Vec<_>>();

        w.writer.write_i32::<LittleEndian>(streamline.len() as i32).unwrap();
        for p in streamline {
            w.write_point_and_scalars(&p, &mut scalars);
        }
        for property in properties {
            w.writer.write_f32::<LittleEndian>(property).unwrap();
        }
        w.real_n_count += 1;
    }
}

impl<'data> Writable for RefTractogramItem<'data> {
    fn write(self, w: &mut Writer) {
        let (streamline, scalars, properties) = self;

        // Transform scalars into a vector of iterators, so we always know where we were
        let mut scalars = scalars.into_iter().map(|v| v.iter().cloned()).collect::<Vec<_>>();

        w.writer.write_i32::<LittleEndian>(streamline.len() as i32).unwrap();
        for p in streamline {
            w.write_point_and_scalars(p, &mut scalars);
        }
        for property in properties {
            w.writer.write_f32::<LittleEndian>(property).unwrap();
        }
        w.real_n_count += 1;
    }
}

impl<'data> Writable for &'data [Point] {
    fn write(self, w: &mut Writer) {
        w.writer.write_i32::<LittleEndian>(self.len() as i32).unwrap();
        let mut scalars: Vec<::std::vec::IntoIter<f32>> = vec![];
        for p in self {
            w.write_point_and_scalars(p, &mut scalars);
        }
        w.real_n_count += 1;
    }
}

impl Writer {
    pub fn new<P: AsRef<Path>>(
        path: P,
        reference: Option<Header>
    ) -> Result<Writer> {
        let f = File::create(path).expect("Can't create new trk file.");
        let mut writer = BufWriter::new(f);

        let header = match reference {
            Some(r) => r,
            None => Header::default()
        };
        header.write(&mut writer)?;

        // We are only interested in the inversed affine
        let affine4 = header.affine4.try_inverse().expect(
            "Unable to inverse 4x4 affine matrix");
        let (affine, translation) = get_affine_and_translation(&affine4);

        Ok(Writer { writer, affine4, affine, translation, real_n_count: 0 })
    }

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
        where I: IntoIterator<Item = Point>
    {
        self.writer.write_i32::<LittleEndian>(len as i32).unwrap();
        let mut scalars: Vec<::std::vec::IntoIter<f32>> = vec![];
        for p in streamline {
            self.write_point_and_scalars(&p, &mut scalars);
        }
        self.real_n_count += 1;
    }

    fn write_point_and_scalars<I>(
        &mut self,
        p: &Point,
        scalars: &mut Vec<I>)
        where I: Iterator<Item = f32>
    {
        let p = self.affine * p + self.translation;
        self.writer.write_f32::<LittleEndian>(p.x).unwrap();
        self.writer.write_f32::<LittleEndian>(p.y).unwrap();
        self.writer.write_f32::<LittleEndian>(p.z).unwrap();

        for scalar in scalars.iter_mut() {
            let scalar = scalar.next().expect("Missing some scalars");
            self.writer.write_f32::<LittleEndian>(scalar).unwrap();
        }
    }
}

// Finally write `n_count`
impl Drop for Writer {
    fn drop(&mut self) {
        CHeader::seek_n_count_field(&mut self.writer).expect(
            "Unable to seek to 'n_count' field before closing trk file.");
        self.writer.write_i32::<LittleEndian>(self.real_n_count).expect(
            "Unable to write 'n_count' field before closing trk file.");
    }
}
