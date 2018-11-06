use std::fmt;
use std::fs::File;
use std::io::{BufReader, BufWriter, Error, ErrorKind, Read, Result, Seek, SeekFrom};
use std::str::from_utf8;

use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt, WriteBytesExt};
use nalgebra::{U3, Vector4};

#[cfg(feature = "use_nifti")] use Affine;
use {Affine4, TrkEndianness};
use orientation::{affine_to_axcodes, axcodes_to_orientations,
                  inverse_orientations_affine, orientations_transform};

pub enum Endianness { Little, Big }

impl fmt::Display for Endianness {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Endianness::Little => write!(f, "< (Little)"),
            Endianness::Big => write!(f, "> (Big)")
        }
    }
}

// http://www.trackvis.org/docs/?subsect=fileformat
#[derive(Clone)]
#[repr(C)]
pub struct CHeader {
    pub id_string: [u8; 6],
    pub dim: [i16; 3],
    pub voxel_size: [f32; 3],
    pub origin: [f32; 3],
    pub n_scalars: i16,
    pub scalar_name: [u8; 200],    // [10][20]
    pub n_properties: i16,
    pub property_name: [u8; 200],  // [10][20]
    pub vox_to_ras: [f32; 16],     // [4][4]
    pub reserved: [u8; 444],
    pub voxel_order: [u8; 4],
    pub pad2: [u8; 4],
    pub image_orientation_patient: [f32; 6],
    pub pad1: [u8; 2],
    pub invert_x: u8,
    pub invert_y: u8,
    pub invert_z: u8,
    pub swap_x: u8,
    pub swap_y: u8,
    pub swap_z: u8,
    pub n_count: i32,
    pub version: i32,
    pub hdr_size: i32
}

// TODO Use size_of::<Header>() when possible
// use std::mem::size_of;
pub const HEADER_SIZE: usize = 1000;

impl CHeader {
    #[cfg(feature = "use_nifti")]
    pub fn from_nifti(
        dim: [u16; 8],
        pixdim: [f32; 8],
        srow_x: [f32; 4],
        srow_y: [f32; 4],
        srow_z: [f32; 4]
    ) -> CHeader {
        let affine = Affine::new(srow_x[0], srow_x[1], srow_x[2],
                                 srow_y[0], srow_y[1], srow_y[2],
                                 srow_z[0], srow_z[1], srow_z[2]);
        let vo = affine_to_axcodes(&affine).into_bytes();
        CHeader {
            dim: [dim[1] as i16, dim[2] as i16, dim[3] as i16],
            voxel_size: [pixdim[1], pixdim[2], pixdim[3]],
            vox_to_ras: [srow_x[0], srow_x[1], srow_x[2], srow_x[3],
                         srow_y[0], srow_y[1], srow_y[2], srow_y[3],
                         srow_z[0], srow_z[1], srow_z[2], srow_z[3],
                         0.0, 0.0, 0.0, 1.0],
            voxel_order: [vo[0], vo[1], vo[2], 0u8],
            ..CHeader::default()
        }
    }

    pub fn seek_n_count_field(f: &mut BufWriter<File>) -> Result<()> {
        let n_count_offset = (HEADER_SIZE - 12) as u64;
        f.seek(SeekFrom::Start(n_count_offset))?;
        Ok(())
    }

    pub fn add_scalar(&mut self, name: &str) -> Result<()> {
        if self.n_scalars > 10 {
            Err(Error::new(ErrorKind::InvalidInput, "Trk header is already full of scalars (10)"))
        } else if name.len() > 20 {
            Err(Error::new(ErrorKind::InvalidInput, "New scalar name must be <= 20 characters."))
        } else if !name.is_ascii() {
            Err(Error::new(ErrorKind::InvalidInput, "New scalar name must be pure ascii."))
        } else {
            let pos = 20 * self.n_scalars as usize;
            self.scalar_name[pos..pos + name.len()].clone_from_slice(name.as_bytes());
            self.n_scalars += 1;
            return Ok(())
        }
    }

    pub fn get_scalars_name(&self) -> Vec<String> {
        read_names(&self.scalar_name, self.n_scalars as usize)
    }

    pub fn get_properties_name(&self) -> Vec<String> {
        read_names(&self.property_name, self.n_properties as usize)
    }

    /// Get affine mapping trackvis voxelmm space to RAS+ mm space
    ///
    /// The streamlines in a trackvis file are in 'voxelmm' space, where the coordinates refer to
    /// the corner of the voxel.
    ///
    /// Compute the affine matrix that will bring them back to RAS+ mm space, where the coordinates
    /// refer to the center of the voxel.
    pub fn get_affine_to_rasmm(&self) -> Affine4 {
        let mut affine = Affine4::identity();

        let scale = Affine4::from_diagonal(&Vector4::new(
            1.0 / self.voxel_size[0],
            1.0 / self.voxel_size[1],
            1.0 / self.voxel_size[2],
            1.0));
        affine = scale * affine;

        let offset = Affine4::new(
            1.0, 0.0, 0.0, -0.5,
            0.0, 1.0, 0.0, -0.5,
            0.0, 0.0, 1.0, -0.5,
            0.0, 0.0, 0.0, 1.0);
        affine = offset * affine;

        let voxel_to_rasmm = Affine4::from_iterator(
            self.vox_to_ras.iter().cloned()).transpose();

        let header_ornt = axcodes_to_orientations(
            from_utf8(&self.voxel_order).unwrap());
        let affine_order = affine_to_axcodes(
            &voxel_to_rasmm.fixed_slice::<U3, U3>(0, 0).into_owned());
        let affine_ornt = axcodes_to_orientations(&affine_order);
        let orientations = orientations_transform(&header_ornt, &affine_ornt);
        let inv = inverse_orientations_affine(&orientations, self.dim);
        affine = inv * affine;

        voxel_to_rasmm * affine
    }

    pub fn read_from_file(path: &str) -> Result<(CHeader, Endianness)> {
        let f = File::open(path).expect("Can't read trk file.");
        let mut reader = BufReader::new(f);
        CHeader::read(&mut reader)
    }

    pub fn read(reader: &mut BufReader<File>) -> Result<(CHeader, Endianness)> {
        reader.seek(SeekFrom::Start(0))?;
        let endianness = test_endianness(reader)?;
        let header = match endianness {
            Endianness::Little => CHeader::read_::<LittleEndian>(reader)?,
            Endianness::Big => CHeader::read_::<BigEndian>(reader)?
        };
        Ok((header, endianness))
    }

    fn read_<E: ByteOrder>(reader: &mut BufReader<File>) -> Result<CHeader> {
        let mut header = CHeader::default();

        reader.read_exact(&mut header.id_string)?;
        for i in &mut header.dim {
            *i = reader.read_i16::<E>()?;
        }
        for f in &mut header.voxel_size {
            *f = reader.read_f32::<E>()?;
        }
        for f in &mut header.origin {
            *f = reader.read_f32::<E>()?;
        }
        header.n_scalars = reader.read_i16::<E>()?;
        reader.read_exact(&mut header.scalar_name)?;
        header.n_properties = reader.read_i16::<E>()?;
        reader.read_exact(&mut header.property_name)?;
        for f in &mut header.vox_to_ras {
            *f = reader.read_f32::<E>()?;
        }
        reader.read_exact(&mut header.reserved)?;
        reader.read_exact(&mut header.voxel_order)?;
        reader.read_exact(&mut header.pad2)?;
        for f in &mut header.image_orientation_patient {
            *f = reader.read_f32::<E>()?;
        }
        reader.read_exact(&mut header.pad1)?;
        header.invert_x = reader.read_u8()?;
        header.invert_y = reader.read_u8()?;
        header.invert_z = reader.read_u8()?;
        header.swap_x = reader.read_u8()?;
        header.swap_y = reader.read_u8()?;
        header.swap_z = reader.read_u8()?;
        header.n_count = reader.read_i32::<E>()?;
        header.version = reader.read_i32::<E>()?;
        header.hdr_size = reader.read_i32::<E>()?;

        Ok(header)
    }

    pub fn write<W: WriteBytesExt>(&self, writer: &mut W) -> Result<()> {
        writer.write(&self.id_string)?;
        for i in &self.dim {
            writer.write_i16::<TrkEndianness>(*i)?;
        }
        for f in &self.voxel_size {
            writer.write_f32::<TrkEndianness>(*f)?;
        }
        for f in &self.origin {
            writer.write_f32::<TrkEndianness>(*f)?;
        }
        writer.write_i16::<TrkEndianness>(self.n_scalars)?;
        writer.write(&self.scalar_name)?;
        writer.write_i16::<TrkEndianness>(self.n_properties)?;
        writer.write(&self.property_name)?;
        for f in &self.vox_to_ras {
            writer.write_f32::<TrkEndianness>(*f)?;
        }
        writer.write(&self.reserved)?;
        writer.write(&self.voxel_order)?;
        writer.write(&self.pad2)?;
        for f in &self.image_orientation_patient {
            writer.write_f32::<TrkEndianness>(*f)?;
        }
        writer.write(&self.pad1)?;
        writer.write_u8(self.invert_x)?;
        writer.write_u8(self.invert_y)?;
        writer.write_u8(self.invert_z)?;
        writer.write_u8(self.swap_x)?;
        writer.write_u8(self.swap_y)?;
        writer.write_u8(self.swap_z)?;
        writer.write_i32::<TrkEndianness>(self.n_count)?;
        writer.write_i32::<TrkEndianness>(self.version)?;
        writer.write_i32::<TrkEndianness>(self.hdr_size)?;

        Ok(())
    }
}

impl Default for CHeader {
    fn default() -> CHeader {
         CHeader {
            id_string: *b"TRACK\0",
            dim: [0, 0, 0],
            voxel_size: [1.0, 1.0, 1.0],
            origin: [0.0, 0.0, 0.0],
            n_scalars: 0,
            scalar_name: [0; 200],
            n_properties: 0,
            property_name: [0; 200],
            vox_to_ras: [1.0, 0.0, 0.0, 0.0,
                         0.0, 1.0, 0.0, 0.0,
                         0.0, 0.0, 1.0, 0.0,
                         0.0, 0.0, 0.0, 1.0],
            reserved: [0; 444],
            voxel_order: [82, 65, 83, 0],
            pad2: [0; 4],
            image_orientation_patient: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            pad1: [0; 2],
            invert_x: 0, invert_y: 0, invert_z: 0,
            swap_x: 0, swap_y: 0, swap_z: 0,
            n_count: 0,
            version: 2,
            hdr_size: HEADER_SIZE as i32
        }
    }
}

/// Returns the endianness used when saving the trk file read by `reader`
///
/// We use `version` to discover the endianness because it's the biggest
/// integer field with the most constrained possible values {1, 2}.
/// Read in LittleEndian, version == 1 or 2.
/// Read in BigEndian, version == 511 or 767
/// Even with hundreds major updates, `version` should be safe.
fn test_endianness(reader: &mut BufReader<File>) -> Result<Endianness> {
    let version_offset = (HEADER_SIZE - 8) as u64;
    reader.seek(SeekFrom::Start(version_offset))?;
    let version = reader.read_i32::<LittleEndian>()?;
    let endianness = if version <= 255 {
        Endianness::Little
    } else {
        Endianness::Big
    };
    reader.seek(SeekFrom::Start(0))?;

    Ok(endianness)
}

/// Returns the names from the [10][20] arrays of bytes.
///
/// Normal case: name\0\0...
/// Special case: name\0{number}\0\0...
fn read_names(names_bytes: &[u8], nb: usize) -> Vec<String> {
    let mut names = Vec::with_capacity(nb);
    for names_byte in names_bytes.chunks(20) {
        if names_byte[0] == 0u8 { break; }

        let idx = names_byte.iter().position(|&e| e == 0u8).unwrap_or(20);
        let name = from_utf8(&names_byte[..idx]).unwrap().to_string();
        if idx < 19 && names_byte[idx + 1] != 0u8 {
            let number = names_byte[idx + 1] - 48;
            for _ in 0..number {
                names.push(name.clone());
            }
        } else {
            names.push(name);
        }
    }

    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scalars() {
        let mut header = CHeader::default();
        header.add_scalar("color_x").unwrap();
        header.add_scalar("color_y").unwrap();

        let mut gt = [0u8; 200];
        gt[..7].clone_from_slice(b"color_x");
        gt[20..27].clone_from_slice(b"color_y");
        assert_eq!(&header.scalar_name[..], &gt[..]);
    }
}
