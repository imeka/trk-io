
extern crate docopt;
extern crate trk_io;

use docopt::Docopt;
use std::path::Path;
use std::str;
use trk_io::CHeader;

static USAGE: &'static str = "
Print a TrackVis (.trk) header in an readable form

Usage:
  trk_header <input> [options]
  trk_header (-h | --help)
  trk_header --version

Options:
  -h --help              Show this screen.
  --version              Show version.
";

fn main() {
    let version = String::from(env!("CARGO_PKG_VERSION"));
    let args = Docopt::new(USAGE)
                      .and_then(|dopt| dopt.version(Some(version)).parse())
                      .unwrap_or_else(|e| e.exit());
    let input = Path::new(args.get_str("<input>"));
    if !input.exists() {
        panic!("Input trk '{:?}' doesn't exist.", input);
    }
    let input = input.to_str()
        .expect("Your input path contains non-UTF-8 cahracters");

    let (header, endianness) = CHeader::read_from_file(input);
    println!("Endianness {}   (not an actual field)", endianness);
    println!("id_string: {:?} ({})",
        header.id_string,
        str::from_utf8(&header.id_string).unwrap());
    println!("dim: {:?}", header.dim);
    println!("voxel_size: {:?}", header.voxel_size);
    println!("origin: {:?}", header.origin);
    println!("n_scalars: {:?}", header.n_scalars);
    for (i, scalar_name) in header.get_scalars_name().iter().enumerate() {
        println!("  {}: {}", i, scalar_name);
    }
    println!("n_properties: {:?}", header.n_properties);
    for (i, property_name) in header.get_properties_name().iter().enumerate() {
        println!("  {}: {}", i, property_name);
    }
    println!("vox_to_ras: {:?}", &header.vox_to_ras[0..4]);
    println!("            {:?}", &header.vox_to_ras[4..8]);
    println!("            {:?}", &header.vox_to_ras[8..12]);
    println!("            {:?}", &header.vox_to_ras[12..16]);
    println!("voxel_order: {:?}", header.voxel_order);
    println!("image_orientation_patient: {:?}",
        header.image_orientation_patient);
    println!("invert: {:?} {:?} {:?}",
        header.invert_x, header.invert_y, header.invert_z);
    println!("swap: {:?} {:?} {:?}",
        header.swap_x, header.swap_y, header.swap_z);
    println!("n_count: {:?}", header.n_count);
    println!("version: {:?}", header.version);
    println!("hdr_size: {:?}", header.hdr_size);
}
