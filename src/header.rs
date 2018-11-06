use std::fs::File;
use std::io::{BufReader, Result};

use byteorder::WriteBytesExt;
#[cfg(feature = "use_nifti")] use nifti::NiftiHeader;

use {Affine, Affine4, Translation};
use affine::get_affine_and_translation;
use cheader::{CHeader, Endianness};

#[derive(Clone)]
pub struct Header {
    c_header: CHeader,
    pub affine4_to_rasmm: Affine4,
    pub affine_to_rasmm: Affine,
    pub translation: Translation,
    pub nb_streamlines: usize,

    pub scalars_name: Vec<String>,
    pub properties_name: Vec<String>
}

impl Header {
    #[cfg(feature = "use_nifti")]
    pub fn from_nifti(h: &NiftiHeader) -> Header {
        let c_header = CHeader::from_nifti(
            h.dim, h.pixdim, h.srow_x, h.srow_y, h.srow_z);
        let affine4 = c_header.get_affine_to_rasmm();
        let (affine, translation) = get_affine_and_translation(&affine4);
        Header {
            c_header,
            affine4_to_rasmm: affine4,
            affine_to_rasmm: affine,
            translation,
            nb_streamlines: 0,
            scalars_name: vec![], properties_name: vec![]
        }
    }

    pub fn add_scalar(&mut self, name: &str) -> Result<()> {
        self.c_header.add_scalar(name)?;
        self.scalars_name.push(name.to_string());
        Ok(())
    }

    pub fn read(reader: &mut BufReader<File>) -> Result<(Header, Endianness)> {
        let (c_header, endianness) = CHeader::read(reader)?;
        let affine4 = c_header.get_affine_to_rasmm();
        let (affine, translation) = get_affine_and_translation(&affine4);
        let nb_streamlines = c_header.n_count as usize;
        let scalars_name = c_header.get_scalars_name();
        let properties_name = c_header.get_properties_name();

        let header = Header {
            c_header,
            affine4_to_rasmm: affine4,
            affine_to_rasmm: affine,
            translation,
            nb_streamlines,
            scalars_name,
            properties_name
        };
        Ok((header, endianness))
    }

    pub fn write<W: WriteBytesExt>(&self, writer: &mut W) -> Result<()> {
        Ok(self.c_header.write(writer)?)
    }
}

impl Default for Header {
    fn default() -> Header {
        Header {
            c_header: CHeader::default(),
            affine4_to_rasmm: Affine4::identity(),
            affine_to_rasmm: Affine::identity(),
            translation: Translation::zeros(),
            nb_streamlines: 0,
            scalars_name: vec![],
            properties_name: vec![]
        }
    }
}

impl PartialEq for Header {
    fn eq(&self, other: &Header) -> bool {
        self.affine_to_rasmm == other.affine_to_rasmm &&
        self.translation == other.translation &&
        self.nb_streamlines == other.nb_streamlines &&
        self.scalars_name == other.scalars_name &&
        self.properties_name == other.properties_name
    }
}
