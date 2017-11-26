
use std::fs::{File};
use std::io::{BufReader, BufWriter, Seek, SeekFrom, Write};
use std::slice::from_raw_parts;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use nalgebra::{U3};

use header::{get_affine, Header, HEADER_SIZE, read_header};
use streamlines::{Streamlines, Point};

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
