
extern crate docopt;
extern crate rand;
extern crate trk_io;

use std::path::Path;
use std::str;

use docopt::Docopt;
use rand::Rng;

use trk_io::{Reader, Writer};

static USAGE: &'static str = "
Subsample a TrackVis (.trk) file

Usage:
  trk_subsampler <input> <output> (--percent=<p> | --number=<n>)
  trk_subsampler (-h | --help)
  trk_subsampler (-v | --version)

Options:
  -p --percent=<p>  Keep only p% of streamlines. Based on rand.
  -n --number=<n>   Keep exactly n streamlines. Deterministic.
  -h --help         Show this screen.
  -v --version      Show version.
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
    let mut writer = Writer::new(
        args.get_str("<output>"), Some(reader.header.clone()));

    if let Ok(percent) = args.get_str("--percent").parse::<f32>() {
        let percent = percent / 100.0;
        let mut rng = rand::thread_rng();

        for streamline in reader.into_iter() {
            if rng.gen::<f32>() < percent {
                writer.write(&streamline);
            }
        }
    } else if let Ok(_nb) = args.get_str("--number").parse::<usize>() {
        panic!("Not implemented yet");
    } else {
        panic!("--percent or --number can't be parsed to a number");
    }
}
