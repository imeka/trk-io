
use byteorder::{WriteBytesExt};

use {Affine, Translation};
use cheader::{CHeader, Endianness};

#[derive(Clone)]
pub struct Header {
    c_header: CHeader,
    pub affine: Affine,
    pub translation: Translation,
    pub nb_streamlines: usize,
    pub scalars_name: Vec<String>,
    pub properties_name: Vec<String>
}

impl Header {
    pub fn read(path: &str) -> (Header, Endianness) {
        let (c_header, endianness) = CHeader::read(path);
        let (affine, translation) = c_header.get_affine();
        let nb_streamlines = c_header.n_count as usize;

        let scalars_name = (0..c_header.n_scalars as usize).map(
            |i| c_header.get_scalar(i)).collect();
        let properties_name = (0..c_header.n_properties as usize).map(
            |i| c_header.get_property(i)).collect();

        let header = Header {
            c_header, affine, translation, nb_streamlines,
            scalars_name, properties_name
        };
        (header, endianness)
    }

    pub fn write<W: WriteBytesExt>(&self, writer: &mut W) {
        self.c_header.write(writer);
    }
}

impl Default for Header {
    fn default() -> Header {
        Header {
            c_header: CHeader::default(),
            affine: Affine::identity(),
            translation: Translation::zeros(),
            nb_streamlines: 0,
            scalars_name: vec![],
            properties_name: vec![]
        }
    }
}

impl PartialEq for Header {
    fn eq(&self, other: &Header) -> bool {
        self.affine == other.affine &&
        self.translation == other.translation &&
        self.nb_streamlines == other.nb_streamlines &&
        self.scalars_name == other.scalars_name &&
        self.properties_name == other.properties_name
    }
}
