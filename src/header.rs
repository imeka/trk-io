
use std::fs::{File};
use std::io::{BufReader, Read};
use std::slice::from_raw_parts;
use std::str::from_utf8;

use byteorder::{WriteBytesExt};
use nalgebra::{Matrix3, Matrix4, RowVector3, Vector4};

type Affine4 = Matrix4<f32>;
pub type Affine = Matrix3<f32>;
pub type Translation = RowVector3<f32>;

// http://www.trackvis.org/docs/?subsect=fileformat
#[repr(C, packed)]
pub struct Header {
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

impl Default for Header {
    fn default() -> Header {
         Header {
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

impl Header {
    pub fn get_scalar(&self, i: usize) -> &str {
        if i >= 10 {
            panic!("There's no more than {} scalars", i);
        }
        let min_ = i * 20;
        let max_ = min_ + 10;
        let name = &self.scalar_name[min_..max_];
        from_utf8(name).expect("get_scalar failed")
    }

    pub fn get_property(&self, i: usize) -> &str {
        if i >= 10 {
            panic!("There's no more than {} properties", i);
        }
        let min_ = i * 20;
        let max_ = min_ + 10;
        let name = &self.property_name[min_..max_];
        from_utf8(name).expect("get_property failed")
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

        // Lotta complicated shits. TODO
        // let header_ornt = from_utf8(&header.voxel_order).unwrap();
        // let affine_ornt = "RAS";
        //let M = Affine::identity();
        //affine = M * affine;

        let voxel_to_rasmm = Affine4::from_iterator(
            self.vox_to_ras.iter().cloned()).transpose();
        affine = voxel_to_rasmm * affine;

        let translation = RowVector3::new(
            affine[12], affine[13], affine[14]);
        // TODO fixed_slice seems better but it's only a reference
        let affine = affine.remove_row(3).remove_column(3);
        (affine, translation)
    }

    pub fn write<W: WriteBytesExt>(&self, writer: &mut W) {
        let bytes = unsafe {
            from_raw_parts(self as *const Header as *const u8, HEADER_SIZE)
        };
        writer.write(bytes).unwrap();
    }
}

pub fn read_header(path: &str) -> Header {
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
