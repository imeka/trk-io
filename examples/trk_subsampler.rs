
extern crate docopt;
extern crate rand;
extern crate trk_io;

use std::path::Path;
use std::str;

use docopt::Docopt;
use rand::Rng;
use rand::SeedableRng;
use rand::FromEntropy;
use rand::rngs::SmallRng;

use trk_io::{Reader, Writer};

static USAGE: &'static str = "
Subsample a TrackVis (.trk) file

Usage:
  trk_subsampler <input> <output> (--percent=<p> | --number=<n>) [--seed=<s>]
  trk_subsampler (-h | --help)
  trk_subsampler (-v | --version)

Options:
  -p --percent=<p>  Keep only p% of streamlines. Based on rand.
  -n --number=<n>   Keep exactly n streamlines. Based on rand.
  -s --seed=<s>     Make randomness deterministic.
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

    let mut rng = match args.get_str("--seed").parse::<u8>() {
        Ok(seed) => SmallRng::from_seed([seed; 16]),
        _        => SmallRng::from_entropy()
    };

    if let Ok(percent) = args.get_str("--percent").parse::<f32>() {
        let percent = percent / 100.0;

        for streamline in reader.into_iter() {
            if rng.gen::<f32>() < percent {
                writer.write(&streamline);
            }
        }
    } else if let Ok(nb) = args.get_str("--number").parse::<usize>() {
        let size = reader.header.nb_streamlines;
        let mut number = nb;

        if number > size {
            println!(
                "The number {} exceed the total number of streamlines: {}. \
                 Saving {} streamlines.",
                number, size, size);
            number = size;
        } else if number == 0 {
            panic!("Saving 0 streamline is not usefull. \
                    Please change the number of steamlines you want.");
        }

        let mut sampled_indices = rand::seq::sample_indices(&mut rng, size, number);
        sampled_indices.sort();

        let mut reader_iter = reader.into_iter();
        let mut indices_iter = sampled_indices.into_iter();
        let mut last: usize = 0;
        for idx in indices_iter {
            let streamline = reader_iter.nth(idx - last).unwrap();
            writer.write(&streamline);
            last = idx + 1;
        }
    } else {
        panic!("--percent or --number can't be parsed to a positive number");
    }
}
