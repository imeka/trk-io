
use std::fs::{File};
use std::io::{BufReader, Read};
use std::slice::from_raw_parts;
use std::str::from_utf8;

use byteorder::{WriteBytesExt};
use nalgebra::{U3, Vector4};

use {Affine, Affine4, Translation};
use orientation::{affine_to_axcodes, axcodes_to_orientations,
                  inverse_orientations_affine, orientations_transform};

pub struct Header {
    c_header: CHeader,
    pub affine: Affine,
    pub translation: Translation,
    pub nb_streamlines: usize,
    pub scalars_name: Vec<String>,
    pub properties_name: Vec<String>
}

impl Header {
    pub fn write<W: WriteBytesExt>(&self, writer: &mut W) {
        self.c_header.write(writer);
    }
}

// http://www.trackvis.org/docs/?subsect=fileformat
#[repr(C, packed)]
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

impl Default for CHeader {
    fn default() -> CHeader {
         CHeader {
            id_string: *b"TRACK\0",
            dim: [1, 1, 1],
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
            image_orientation_patient: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            pad1: [0; 2],
            invert_x: 0, invert_y: 0, invert_z: 0,
            swap_x: 0, swap_y: 0, swap_z: 0,
            n_count: 0,
            version: 2,
            hdr_size: HEADER_SIZE as i32
        }
    }
}

impl CHeader {
    pub fn get_scalar(&self, i: usize) -> String {
        if i >= 10 {
            panic!("There's no more than {} scalars", i);
        }
        let min_ = i * 20;
        let max_ = min_ + 10;
        let name = &self.scalar_name[min_..max_];
        from_utf8(name).expect("get_scalar failed").to_string()
    }

    pub fn get_property(&self, i: usize) -> String {
        if i >= 10 {
            panic!("There's no more than {} properties", i);
        }
        let min_ = i * 20;
        let max_ = min_ + 10;
        let name = &self.property_name[min_..max_];
        from_utf8(name).expect("get_property failed").to_string()
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

    pub fn write<W: WriteBytesExt>(&self, writer: &mut W) {
        let bytes = unsafe {
            from_raw_parts(self as *const CHeader as *const u8, HEADER_SIZE)
        };
        writer.write(bytes).unwrap();
    }
}

pub fn read_c_header(path: &str) -> CHeader {
    let f = File::open(path).expect("Can't read trk file.");
    let mut reader = BufReader::new(f);
    unsafe {
        let mut s = ::std::mem::uninitialized();
        let buffer = ::std::slice::from_raw_parts_mut(
            &mut s as *mut _ as *mut u8, HEADER_SIZE);
        match reader.read_exact(buffer) {
            Ok(()) => s,
            Err(_) => {
                ::std::mem::forget(s);
                panic!("Can't read header from trk file.");
            }
        }
    }
}

pub fn read_header(path: &str) -> Header {
    let c_header = read_c_header(path);
    let (affine, translation) = c_header.get_affine();
    let nb_streamlines = c_header.n_count as usize;

    let mut scalars_name = Vec::with_capacity(c_header.n_scalars as usize);
    for i in 0..scalars_name.capacity() {
        scalars_name.push(c_header.get_scalar(i));
    }

    let mut properties_name = Vec::with_capacity(c_header.n_properties as usize);
    for i in 0..properties_name.capacity() {
        properties_name.push(c_header.get_property(i));
    }

    Header {
        c_header, affine, translation, nb_streamlines,
        scalars_name, properties_name
    }
}
