use std::fs::File;
use std::io::{BufReader, Result};

use byteorder::WriteBytesExt;
#[cfg(feature = "nifti_images")]
use nifti::NiftiHeader;

use affine::get_affine_and_translation;
use cheader::{CHeader, Endianness};
use {Affine, Affine4, Translation};

#[derive(Clone)]
pub struct Header {
    c_header: CHeader,
    pub affine4_to_rasmm: Affine4,
    pub affine_to_rasmm: Affine,
    pub translation: Translation,
    pub nb_streamlines: usize,

    pub scalars_name: Vec<String>,
    pub properties_name: Vec<String>,
}

impl Header {
    #[cfg(feature = "nifti_images")]
    pub fn from_nifti(h: &NiftiHeader) -> Header {
        let c_header = CHeader::from_nifti(h.dim, h.pixdim, h.srow_x, h.srow_y, h.srow_z);
        let affine4 = c_header.get_affine_to_rasmm();
        let (affine, translation) = get_affine_and_translation(&affine4);
        Header {
            c_header,
            affine4_to_rasmm: affine4,
            affine_to_rasmm: affine,
            translation,
            nb_streamlines: 0,
            scalars_name: vec![],
            properties_name: vec![],
        }
    }

    /// Clear all scalars and properties from `self`.
    pub fn clear_scalars_and_properties(&mut self) {
        self.clear_scalars();
        self.clear_properties();
    }

    /// Clear all scalars from `self`.
    pub fn clear_scalars(&mut self) {
        self.scalars_name.clear();
        self.c_header.clear_scalars();
    }

    /// Clear all properties from `self`.
    pub fn clear_properties(&mut self) {
        self.properties_name.clear();
        self.c_header.clear_properties();
    }

    /// Clear all scalars and properties from `self` and copy scalars and properties from `rhs`.
    pub fn copy_scalars_and_properties(&mut self, rhs: &Self) {
        self.copy_scalars(rhs);
        self.copy_properties(rhs);
    }

    /// Clear all scalars from `self` and copy scalars from `rhs`.
    pub fn copy_scalars(&mut self, rhs: &Self) {
        self.clear_scalars();
        for scalar in &rhs.scalars_name {
            self.add_scalar(scalar).unwrap(); // Can't fail
        }
    }

    /// Clear all properties from `self` and copy properties from `rhs`.
    pub fn copy_properties(&mut self, rhs: &Self) {
        self.clear_properties();
        for property in &rhs.properties_name {
            self.add_property(property).unwrap(); // Can't fail
        }
    }

    pub fn add_scalar(&mut self, name: &str) -> Result<()> {
        self.c_header.add_scalar(name)?;
        self.scalars_name.push(name.to_string());
        Ok(())
    }

    pub fn add_property(&mut self, name: &str) -> Result<()> {
        self.c_header.add_property(name)?;
        self.properties_name.push(name.to_string());
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
            properties_name,
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
            properties_name: vec![],
        }
    }
}

impl PartialEq for Header {
    fn eq(&self, other: &Header) -> bool {
        self.affine_to_rasmm == other.affine_to_rasmm
            && self.translation == other.translation
            && self.nb_streamlines == other.nb_streamlines
            && self.scalars_name == other.scalars_name
            && self.properties_name == other.properties_name
    }
}
