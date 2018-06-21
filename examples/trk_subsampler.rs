
extern crate docopt;
extern crate rand;
extern crate trk_io;

use std::path::Path;
use std::str;

use docopt::Docopt;
use rand::Rng;
use rand::SeedableRng;

use trk_io::{Reader, Writer};

static USAGE: &'static str = "
Subsample a TrackVis (.trk) file

Usage:
  trk_subsampler <input> <output> (--percent=<p> | --number=<n>) [--seed=<s>]
  trk_subsampler (-h | --help)
  trk_subsampler (-v | --version)

Options:
  -p --percent=<p>  Keep only p% of streamlines. Based on rand.
  -n --number=<n>   Keep exactly n streamlines. Deterministic.
  -s --seed=<s>     Make randomness deterministic [default: 42].
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
        args.get_str("<output>"), Some(reader.header.clone())).unwrap();

    if let Ok(percent) = args.get_str("--percent").parse::<f32>() {
        let percent = percent / 100.0;
        let mut rng = rand::weak_rng();

        for streamline in reader.into_iter() {
            if rng.gen::<f32>() < percent {
                writer.write(&streamline);
            }
        }
    } else if let Ok(nb) = args.get_str("--number").parse::<usize>() {
        if let Ok(seed) = args.get_str("--seed").parse::<u8>() {
            let size = reader.header.nb_streamlines;
            let mut rng = rand::XorShiftRng::from_seed([seed; 16]);

            let mut sampled_indices = rand::seq::sample_indices(&mut rng, size, nb);
            sampled_indices.sort();

            let mut iter_indices = sampled_indices.into_iter();
            let mut index = iter_indices.next();

            for (idx, streamline) in reader.into_iter().enumerate() {
                if let Some(index_to_save) = index {
                    if idx == index_to_save {
                        writer.write(&streamline);
                        index = iter_indices.next();
                    }
                } else {
                    break;
                }
            }
        } else {
            panic!("--number need a seed to be deterministic.");
        }
    } else {
        panic!("--percent or --number can't be parsed to a number");
    }
}
