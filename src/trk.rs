
use std::fs::{File};
use std::io::{BufReader, BufWriter, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use header::{Header, HEADER_SIZE, read_header};
use streamlines::{Streamlines, Point};

pub fn read_streamlines(path: &str) -> (Header, Streamlines) {
    let header = read_header(path);
    let (affine, translation) = header.get_affine();

    let mut f = File::open(path).expect("Can't read trk file.");
    f.seek(SeekFrom::Start(HEADER_SIZE as u64)).unwrap();
    let mut reader = BufReader::new(f);

    let chunk_by = 3 + header.n_scalars as usize;
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

            for floats in float_buffer.chunks(chunk_by) {
                let p = Point::new(floats[0], floats[1], floats[2]);
                v.push((p * affine) + translation);
            }

            // Ignore properties for now
            for _ in 0..header.n_properties {
                reader.read_f32::<LittleEndian>().unwrap();
            }
        }
        else { break; }
    }

    (header, Streamlines::new(affine, translation, lengths, v))
}

pub fn write_streamlines(streamlines: &Streamlines, path: &str) {
    let affine = streamlines.affine.try_inverse().unwrap();
    let translation = streamlines.translation;

    let f = File::create(path).expect("Can't create new trk file.");
    let mut writer = BufWriter::new(f);

    let header = Header {
        n_count: streamlines.lengths.len() as i32,
        ..Header::default()
    };
    header.write(&mut writer);

    for streamline in streamlines {
        writer.write_i32::<LittleEndian>(streamline.len() as i32).unwrap();
        for p in streamline {
            let p = (p - translation) * affine;
            writer.write_f32::<LittleEndian>(p.x).unwrap();
            writer.write_f32::<LittleEndian>(p.y).unwrap();
            writer.write_f32::<LittleEndian>(p.z).unwrap();
        }
    }
}
