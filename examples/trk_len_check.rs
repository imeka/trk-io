use std::{fs::File, path::Path, str};

use docopt::Docopt;

use trk_io::Reader;

static USAGE: &'static str = "
Binary to test that reserving a number of points, scalars and properties is
possible and works 100% of the time.

Usage:
  trk_len_check <input> [options]
  trk_len_check (-h | --help)
  trk_len_check (-v | --version)

Options:
  -h --help     Show this screen.
  -v --version  Show version.
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

    let f = File::open(args.get_str("<input>")).expect("Can't read trk file.");
    let metadata = f.metadata().unwrap();
    let nb_bytes = metadata.len() as usize;

    let reader = Reader::new(args.get_str("<input>")).expect("Read header");
    let header = &reader.header;
    let nb_scalars = header.scalars_name.len();
    let nb_properties = header.properties_name.len();
    let nb_streamlines = header.nb_streamlines;
    let header_size = header.raw_header().hdr_size as usize;

    let nb_f32s = (nb_bytes - header_size) / 4;
    let guessed_nb_properties = nb_streamlines * nb_properties;

    // We remove the nb_points count and the properties, se we only have the points and scalars
    let nb_f32s = nb_f32s - nb_streamlines - guessed_nb_properties;
    let guessed_nb_points = nb_f32s / (3 + nb_scalars);
    let guessed_nb_scalars = nb_f32s - (3 * guessed_nb_points);

    let mut real_nb_points = 0;
    let mut real_nb_scalars = 0;
    let mut real_nb_properties = 0;
    for (streamline, scalars, properties) in reader.into_iter() {
        real_nb_points += streamline.len();
        for s in &scalars {
            real_nb_scalars += s.len();
        }
        real_nb_properties += properties.len();
    }

    assert_eq!(guessed_nb_points, real_nb_points);
    assert_eq!(guessed_nb_scalars, real_nb_scalars);
    assert_eq!(guessed_nb_properties, real_nb_properties);
}
