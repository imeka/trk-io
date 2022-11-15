use std::{fs::File, io::BufReader};

use anyhow::{Context, Result};
use docopt::Docopt;

use trk_io::CHeader;

static USAGE: &'static str = "
Print a TrackVis (.trk) header in an readable form

Usage:
  trk_header <input> [options]
  trk_header (-h | --help)
  trk_header (-v | --version)

Options:
  -a --all       Also print computed fields (endianness, affine, etc.)
  -h --help      Show this screen.
  -v --version   Show version.
";

fn main() -> Result<()> {
    let version = String::from(env!("CARGO_PKG_VERSION"));
    let args = Docopt::new(USAGE)
        .and_then(|dopt| dopt.version(Some(version)).parse())
        .unwrap_or_else(|e| e.exit());
    let print_all = args.get_bool("--all");

    let path = args.get_str("<input>");
    let mut reader =
        BufReader::new(File::open(path).with_context(|| format!("Failed to load {:?}", path))?);
    let (header, endianness) = CHeader::read(&mut reader)?;

    if print_all {
        println!("---------- Actual fields ----------");
    }
    println!(
        "id_string: {:?} ({})",
        header.id_string,
        std::str::from_utf8(&header.id_string).unwrap()
    );
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
    println!(
        "voxel_order: {:?} ({})",
        header.voxel_order,
        std::str::from_utf8(&header.voxel_order).unwrap()
    );
    println!("image_orientation_patient: {:?}", header.image_orientation_patient);
    println!("invert: {:?} {:?} {:?}", header.invert_x, header.invert_y, header.invert_z);
    println!("swap: {:?} {:?} {:?}", header.swap_x, header.swap_y, header.swap_z);
    println!("n_count: {:?}", header.n_count);
    println!("version: {:?}", header.version);
    println!("hdr_size: {:?}", header.hdr_size);

    if print_all {
        let to_rasmm = header.get_affine_to_rasmm();
        let to_trackvis = to_rasmm.try_inverse().expect("Can't inverse affine");
        println!("\n---------- Computed fields ----------");
        println!("Endianness {}", endianness);
        print!("to rasmm {}", to_rasmm);
        print!("to to_trackvis {}", to_trackvis);
    }

    Ok(())
}
