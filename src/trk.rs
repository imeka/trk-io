
extern crate byteorder;
extern crate nalgebra;

// use std::mem::size_of;
use std::fs::{File};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::slice::from_raw_parts;
use std::str::from_utf8;

use trk::byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use trk::nalgebra::{Matrix4, U3, Vector4};

use streamlines::{Streamlines, Point};

// http://www.trackvis.org/docs/?subsect=fileformat
#[repr(C, packed)]
struct Header {
    id_string: [u8; 6],
    dim: [i16; 3],
    voxel_size: [f32; 3],
    origin: [f32; 3],
    n_scalars: i16,
    scalar_name: [u8; 200],    // [10][20]
    n_properties: i16,
    property_name: [u8; 200],  // [10][20]
    vox_to_ras: [f32; 16],     // [4][4]
    reserved: [u8; 444],
    voxel_order: [u8; 4],
    pad2: [u8; 4],
    image_orientation_patient: [f32; 6],
    pad1: [u8; 2],
    invert_x: u8, invert_y: u8, invert_z: u8,
    swap_x: u8, swap_y: u8, swap_z: u8,
    n_count: i32,
    version: i32,
    hdr_size: i32
}

const HEADER_SIZE: usize = 1000;  // size_of::<Header>();

impl Header {
    fn get_scalar(&self, i: usize) -> &str {
        if i >= 10 {
            panic!("");
        }
        let min_ = i * 20;
        let max_ = min_ + 10;
        let name = &self.scalar_name[min_..max_];
        from_utf8(name).expect("get_scalar failed")
    }

    fn get_property(&self, i: usize) -> &str {
        if i >= 10 {
            panic!("");
        }
        let min_ = i * 20;
        let max_ = min_ + 10;
        let name = &self.property_name[min_..max_];
        from_utf8(name).expect("get_property failed")
    }

    pub fn print(&self) {
        println!("id_string: {:?}", &self.id_string);
        println!("dim: {:?}", self.dim);
        println!("voxel_size: {:?}", self.voxel_size);
        println!("origin: {:?}", self.origin);
        println!("n_scalars: {:?}", self.n_scalars);
        for i in 0..self.n_scalars {
            println!("scalar_name {}: {}",
                i, self.get_scalar(i as usize));
        }
        println!("n_properties: {:?}", self.n_properties);
        for i in 0..self.n_properties {
            println!("property_name {}: {}",
                i, self.get_property(i as usize));
        }
        println!("vox_to_ras: {:?}", self.vox_to_ras);
        println!("voxel_order: {:?}", &self.voxel_order);
        println!("image_orientation_patient: {:?}",
                    self.image_orientation_patient);
        println!("invert: {:?} {:?} {:?}",
                    self.invert_x, self.invert_y, self.invert_z);
        println!("swap: {:?} {:?} {:?}",
                    self.swap_x, self.swap_y, self.swap_z);
        println!("n_count: {:?}", self.n_count);
        println!("version: {:?}", self.version);
        println!("hdr_size: {:?}", self.hdr_size);
    }
}

fn read_header(path: &str) -> Header {
    let f = File::open(path).expect("QQQ");
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
    header.print();

    let affine = get_affine(&header);
    println!("{}", affine);

    let mut f = File::open(path).expect("QQQ");
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
                let p = Point {
                    x: x * affine[0] + y * affine[4] + z * affine[8] + affine[12],
                    y: x * affine[1] + y * affine[5] + z * affine[9] + affine[13],
                    z: x * affine[2] + y * affine[6] + z * affine[10] + affine[14]
                };
                v.push(p);
                idx += 3 + header.n_scalars as usize;
            }
        }
        else { break; }
    }

    println!("Nm. points: {}", v.len());
    println!("Nm. streamlines: {}", lengths.len());

    Streamlines::new(affine, lengths, v)
}

pub fn write_streamlines(streamlines: &Streamlines, path: &str) {
    let t_x = streamlines.affine[12];
    let t_y = streamlines.affine[13];
    let t_z = streamlines.affine[14];
    let affine = streamlines.affine.fixed_slice::<U3, U3>(0, 0)
        .try_inverse().unwrap();
    println!("{}", affine);

    let f = File::create(path).expect("QQQ");
    let mut writer = BufWriter::new(f);

    {
        let header = Header {
            id_string: [84, 82, 65, 67, 75, 0],
            dim: [1, 1, 1],                         // From header
            voxel_size: [1.0, 1.0, 1.0],            // From header
            origin: [0.0, 0.0, 0.0],                // From header
            n_scalars: 0,                           // From header
            scalar_name: [0; 200],                  // From header
            n_properties: 0,                        // From header
            property_name: [0; 200],                // From header
            vox_to_ras: [1.0, 0.0, 0.0, 0.0,        // From header
                         0.0, 1.0, 0.0, 0.0,
                         0.0, 0.0, 1.0, 0.0,
                         0.0, 0.0, 0.0, 1.0],
            reserved: [0; 444],
            voxel_order: [82, 65, 83, 0],           // From header
            pad2: [0; 4],
            image_orientation_patient:              // From header
                [1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            pad1: [0; 2],
            invert_x: 0, invert_y: 0, invert_z: 0,  // From header
            swap_x: 0, swap_y: 0, swap_z: 0,        // From header
            n_count: streamlines.lengths.len() as i32,
            version: 2,
            hdr_size: 1000
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
            let t = Point {
                x: raw.x - t_x,
                y: raw.y - t_y,
                z: raw.z - t_z
            };
            // Then transform point with affine
            let p = Point {
                x: t.x * affine[0] + t.y * affine[3] + t.z * affine[6],
                y: t.x * affine[1] + t.y * affine[4] + t.z * affine[7],
                z: t.x * affine[2] + t.y * affine[5] + t.z * affine[8]
            };
            writer.write_f32::<LittleEndian>(p.x).unwrap();
            writer.write_f32::<LittleEndian>(p.y).unwrap();
            writer.write_f32::<LittleEndian>(p.z).unwrap();

            idx += 1;
        }
    }
}
