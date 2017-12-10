
use std::fmt;
use std::fs::{File};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom};
use std::slice::from_raw_parts;
use std::str::from_utf8;

use byteorder::{BigEndian, ByteOrder, LittleEndian,
                ReadBytesExt, WriteBytesExt};
use nalgebra::{U3, Vector4};

use {Affine, Affine4, Translation};
use orientation::{affine_to_axcodes, axcodes_to_orientations,
                  inverse_orientations_affine, orientations_transform};

pub enum Endianness { Little, Big }

impl fmt::Display for Endianness {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Endianness::Little => write!(f, "<"),
            Endianness::Big => write!(f, ">")
        }
    }
}

// http://www.trackvis.org/docs/?subsect=fileformat
#[derive(Clone)]
#[repr(C)]
pub struct CHeader {
    pub id_string: [u8; 6],
    pub dim: [i16; 3],                              // *
    pub voxel_size: [f32; 3],                       // *
    pub origin: [f32; 3],                           // *
    pub n_scalars: i16,                             // *
    pub scalar_name: [u8; 200],    // [10][20]      // *
    pub n_properties: i16,                          // *
    pub property_name: [u8; 200],  // [10][20]      // *
    pub vox_to_ras: [f32; 16],     // [4][4]        // *
    pub reserved: [u8; 444],
    pub voxel_order: [u8; 4],                       // *
    pub pad2: [u8; 4],
    pub image_orientation_patient: [f32; 6],        // *
    pub pad1: [u8; 2],
    pub invert_x: u8,                               // *
    pub invert_y: u8,
    pub invert_z: u8,
    pub swap_x: u8,                                 // *
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
    pub fn seek_n_count_field(f: &mut BufWriter<File>) {
        let n_count_offset = (HEADER_SIZE - 12) as u64;
        f.seek(SeekFrom::Start(n_count_offset)).unwrap();
    }

    pub fn seek_end(f: &mut File) {
        f.seek(SeekFrom::Start(HEADER_SIZE as u64)).unwrap();
    }

    pub fn get_scalars_name(&self) -> Vec<String> {
        read_names(&self.scalar_name, self.n_scalars as usize)
    }

    pub fn get_properties_name(&self) -> Vec<String> {
        read_names(&self.property_name, self.n_properties as usize)
    }

    pub fn get_affine(&self) -> (Affine, Translation) {
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

        affine = voxel_to_rasmm * affine;

        let translation = Translation::new(
            affine[12], affine[13], affine[14]);
        let affine = affine.fixed_slice::<U3, U3>(0, 0).into_owned();
        (affine, translation)
    }

    pub fn read(path: &str) -> (CHeader, Endianness) {
        let f = File::open(path).expect("Can't read trk file.");
        let mut reader = BufReader::new(f);
        let endianness = test_endianness(&mut reader);
        let header = match endianness {
            Endianness::Little => CHeader::read_::<LittleEndian>(&mut reader),
            Endianness::Big => CHeader::read_::<BigEndian>(&mut reader)
        };
        (header, endianness)
    }

    fn read_<E: ByteOrder>(reader: &mut BufReader<File>) -> CHeader {
        let mut header = CHeader::default();

        reader.read_exact(&mut header.id_string).unwrap();
        for i in &mut header.dim {
            *i = reader.read_i16::<E>().unwrap();
        }
        for f in &mut header.voxel_size {
            *f = reader.read_f32::<E>().unwrap();
        }
        for f in &mut header.origin {
            *f = reader.read_f32::<E>().unwrap();
        }
        header.n_scalars = reader.read_i16::<E>().unwrap();
        reader.read_exact(&mut header.scalar_name).unwrap();
        header.n_properties = reader.read_i16::<E>().unwrap();
        reader.read_exact(&mut header.property_name).unwrap();
        for f in &mut header.vox_to_ras {
            *f = reader.read_f32::<E>().unwrap();
        }
        reader.read_exact(&mut header.reserved).unwrap();
        reader.read_exact(&mut header.voxel_order).unwrap();
        reader.read_exact(&mut header.pad2).unwrap();
        for f in &mut header.image_orientation_patient {
            *f = reader.read_f32::<E>().unwrap();
        }
        reader.read_exact(&mut header.pad1).unwrap();
        header.invert_x = reader.read_u8().unwrap();
        header.invert_y = reader.read_u8().unwrap();
        header.invert_z = reader.read_u8().unwrap();
        header.swap_x = reader.read_u8().unwrap();
        header.swap_y = reader.read_u8().unwrap();
        header.swap_z = reader.read_u8().unwrap();
        header.n_count = reader.read_i32::<E>().unwrap();
        header.version = reader.read_i32::<E>().unwrap();
        header.hdr_size = reader.read_i32::<E>().unwrap();

        header
    }

    pub fn write<W: WriteBytesExt>(&self, writer: &mut W) {
        // Because we don't handle scalars and properties for now, we must
        // erase them from the header. We can't actually erase them without
        // being `mut` though, so we write a modified copy.
        let header = CHeader {
            n_scalars: 0,
            scalar_name: [0; 200],
            n_properties: 0,
            property_name: [0; 200],
            ..self.clone()
        };

        let bytes = unsafe {
            from_raw_parts(&header as *const CHeader as *const u8, HEADER_SIZE)
        };
        writer.write(bytes).unwrap();
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
fn test_endianness(reader: &mut BufReader<File>) -> Endianness {
    let version_offset = (HEADER_SIZE - 8) as u64;
    reader.seek(SeekFrom::Start(version_offset)).unwrap();
    let version = reader.read_i32::<LittleEndian>().unwrap();
    let endianness = if version <= 255 {
        Endianness::Little
    } else {
        Endianness::Big
    };
    reader.seek(SeekFrom::Start(0)).unwrap();

    endianness
}

fn read_names(names_bytes: &[u8], nb: usize) -> Vec<String> {
    if nb == 0 {
        return vec![];
    }

    let mut names = Vec::with_capacity(nb);
    for names_byte in names_bytes.chunks(20) {
        for (i, b) in names_byte.iter().enumerate() {
            if *b == 0u8 {
                let name = from_utf8(&names_byte[..i]).unwrap().to_string();
                if names_byte[i + 1] == 0u8 {
                    // Normal case where: name\0\0...
                    names.push(name);
                } else {
                    // Special case where: name\0{number}\0\0...
                    let number = names_byte[i + 1] - 48;
                    for _ in 0..number {
                        names.push(name.clone());
                    }
                }
                break;
            }
        }
        if names.len() == nb { break; }
    }

    names
}
