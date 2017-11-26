
use std::fs::{File};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::slice::from_raw_parts;
use std::str::from_utf8;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use nalgebra::{Matrix4, U3, Vector4};

use streamlines::{Streamlines, Point};

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
const HEADER_SIZE: usize = 1000;

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

fn get_affine(header: &Header) -> Matrix4<f32> {
    let mut affine = Matrix4::identity();

    let scale = Matrix4::from_diagonal(&Vector4::new(
        1.0 / header.voxel_size[0],
        1.0 / header.voxel_size[1],
        1.0 / header.voxel_size[2],
        1.0));
    affine = scale * affine;

    let offset = Matrix4::new(
        1.0, 0.0, 0.0, -0.5,
        0.0, 1.0, 0.0, -0.5,
        0.0, 0.0, 1.0, -0.5,
        0.0, 0.0, 0.0, 1.0);
    affine = offset * affine;

    // Lotta complicated shits. TODO
    // let header_ornt = from_utf8(&header.voxel_order).unwrap();
    // let affine_ornt = "RAS";
    //let M = Matrix4::<f32>::identity();
    //affine = M * affine;

    let voxel_to_rasmm = Matrix4::from_iterator(
        header.vox_to_ras.iter().cloned()).transpose();
    affine = voxel_to_rasmm * affine;

    affine
}

pub fn read_streamlines(path: &str) -> Streamlines {
    let header = read_header(path);
    let affine = get_affine(&header);
    println!("{}", affine);

    let mut f = File::open(path).expect("Can't read trk file.");
    f.seek(SeekFrom::Start(HEADER_SIZE as u64)).unwrap();
    let mut reader = BufReader::new(f);

    let mut float_buffer = vec![0f32; 300];
    let mut v = Vec::with_capacity(3000);
    let mut lengths = Vec::new();
    loop {
        if let Ok(nb_points) = reader.read_i32::<LittleEndian>() {
            lengths.push(nb_points as usize);

            // Vec::resize never decreases capacity, it can only increase it
            // so there won't be any useless allocation.
            let nb_floats = nb_points * (3 + header.n_scalars as i32);
            float_buffer.resize(nb_floats as usize, 0.0);
            unsafe {
                reader.read_f32_into_unchecked::<LittleEndian>(
                    float_buffer.as_mut_slice()).unwrap();
            }

            let mut idx: usize = 0;
            for _ in 0..nb_points {
                let x = float_buffer[idx];
                let y = float_buffer[idx + 1];
                let z = float_buffer[idx + 2];

                // Transform point with affine
                let p = Point::new(
                    x * affine[0] + y * affine[4] + z * affine[8] + affine[12],
                    x * affine[1] + y * affine[5] + z * affine[9] + affine[13],
                    x * affine[2] + y * affine[6] + z * affine[10] + affine[14]
                );
                v.push(p);
                idx += 3 + header.n_scalars as usize;
            }
        }
        else { break; }
    }

    println!("Nb. points: {}", v.len());
    println!("Nb. streamlines: {}", lengths.len());

    Streamlines::new(affine, lengths, v)
}

pub fn write_streamlines(streamlines: &Streamlines, path: &str) {
    let t_x = streamlines.affine[12];
    let t_y = streamlines.affine[13];
    let t_z = streamlines.affine[14];
    let affine = streamlines.affine.fixed_slice::<U3, U3>(0, 0)
        .try_inverse().unwrap();
    println!("{}", affine);

    let f = File::create(path).expect("Can't create new trk file.");
    let mut writer = BufWriter::new(f);

    {
        let header = Header {
            n_count: streamlines.lengths.len() as i32,
            ..Header::default()
        };
        let bytes = unsafe {
            from_raw_parts(&header as *const Header as *const u8, HEADER_SIZE)
        };
        writer.write(bytes).unwrap();
    }

    let mut idx: usize = 0;
    for length in &streamlines.lengths {
        writer.write_i32::<LittleEndian>(*length as i32).unwrap();
        for _ in 0..*length {
            let raw = &streamlines.data[idx];

            // First, translate point
            let t = Point::new(
                raw.x - t_x,
                raw.y - t_y,
                raw.z - t_z);
            // Then transform point with affine
            let p = Point::new(
                t.x * affine[0] + t.y * affine[3] + t.z * affine[6],
                t.x * affine[1] + t.y * affine[4] + t.z * affine[7],
                t.x * affine[2] + t.y * affine[5] + t.z * affine[8]
            );
            writer.write_f32::<LittleEndian>(p.x).unwrap();
            writer.write_f32::<LittleEndian>(p.y).unwrap();
            writer.write_f32::<LittleEndian>(p.z).unwrap();

            idx += 1;
        }
    }
}
