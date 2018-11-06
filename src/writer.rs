use std::fs::File;
use std::io::{BufWriter, Result};
use std::path::Path;

use byteorder::WriteBytesExt;

use affine::get_affine_and_translation;
use tractogram::{Point, RefTractogramItem, Tractogram, TractogramItem};
use {Affine, Affine4, CHeader, Header, Translation, TrkEndianness};

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
    real_n_count: i32,
    nb_scalars: usize,
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

        let header = match reference {
            Some(r) => r,
            None => Header::default(),
        };
        header.write(&mut writer)?;
        let nb_scalars = header.scalars_name.len();

        // We are only interested in the inversed affine
        let affine4 =
            header.affine4_to_rasmm.try_inverse().expect("Unable to inverse 4x4 affine matrix");
        let (affine, translation) = get_affine_and_translation(&affine4);

        Ok(Writer { writer, affine4, affine, translation, real_n_count: 0, nb_scalars })
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
