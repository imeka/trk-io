extern crate docopt;
extern crate trk_io;

use std::path::Path;
use std::str;

use docopt::Docopt;

use trk_io::{Reader, Writer};

static USAGE: &'static str = "
Color a TrackVis (.trk) file

Usage:
  trk_color uniform <r> <g> <b> <input> <output> [options]
  trk_color (-h | --help)
  trk_color (-v | --version)

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

    let reader = Reader::new(args.get_str("<input>")).expect("Read header");
    let mut header = reader.header.clone();
    header.add_scalar("color_x").unwrap();
    header.add_scalar("color_y").unwrap();
    header.add_scalar("color_z").unwrap();

    let mut writer = Writer::new(args.get_str("<output>"), Some(header)).unwrap();

    if args.get_bool("uniform") {
        let r = args.get_str("<r>").parse::<f32>().unwrap();
        let g = args.get_str("<g>").parse::<f32>().unwrap();
        let b = args.get_str("<b>").parse::<f32>().unwrap();
        for (streamline, mut scalars, properties) in reader.into_iter() {
            scalars.push(vec![r; streamline.len()]);
            scalars.push(vec![g; streamline.len()]);
            scalars.push(vec![b; streamline.len()]);

            writer.write((streamline, scalars, properties));
        }
    }
}
