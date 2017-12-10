
use byteorder::{WriteBytesExt};

use {Affine, ArraySequence, Translation};
use cheader::{CHeader, Endianness};

type Scalar = (String, ArraySequence<f32>);
type Property = (String, Vec<f32>);

#[derive(Clone)]
pub struct Header {
    c_header: CHeader,
    pub affine: Affine,
    pub translation: Translation,
    pub nb_streamlines: usize,

    pub scalars: Vec<Scalar>,
    pub properties: Vec<Property>
}

impl Header {
    pub fn read(path: &str) -> (Header, Endianness) {
        let (c_header, endianness) = CHeader::read(path);
        let (affine, translation) = c_header.get_affine();
        let nb_streamlines = c_header.n_count as usize;
        let scalars = c_header.get_scalars_name().into_iter().map(
            |scalar| (scalar, ArraySequence::empty())
        ).collect();
        let properties = c_header.get_properties_name().into_iter().map(
            |property| (property, vec![])
        ).collect();

        let header = Header {
            c_header, affine, translation, nb_streamlines, scalars, properties
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
            scalars: vec![],
            properties: vec![]
        }
    }
}

impl PartialEq for Header {
    fn eq(&self, other: &Header) -> bool {
        self.affine == other.affine &&
        self.translation == other.translation &&
        self.nb_streamlines == other.nb_streamlines &&
        self.scalars == other.scalars &&
        self.properties == other.properties
    }
}
