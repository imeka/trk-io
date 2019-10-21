extern crate docopt;
extern crate rand;
extern crate trk_io;

use std::path::Path;
use std::str;

use docopt::Docopt;
use rand::{rngs::SmallRng, FromEntropy, Rng, SeedableRng};

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

fn sampling_write(writer: &mut Writer, reader: Reader, number: usize, rng: &mut SmallRng) {
    let mut sampled_indices =
        rand::seq::index::sample(rng, reader.header.nb_streamlines, number).into_vec();
    sampled_indices.sort();

    let mut reader_iter = reader.into_iter();
    let mut last = 0;
    for idx in sampled_indices {
        writer.write(reader_iter.nth(idx - last).unwrap());
        last = idx + 1;
    }
}

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
    let mut writer = Writer::new(args.get_str("<output>"), Some(reader.header.clone())).unwrap();

    let mut rng = match args.get_str("--seed").parse::<u8>() {
        Ok(seed) => SmallRng::from_seed([seed; 16]),
        Err(_) => SmallRng::from_entropy(),
    };

    if let Ok(percent) = args.get_str("--percent").parse::<f32>() {
        let percent = percent / 100.0;
        for streamline in reader.into_iter() {
            if rng.gen::<f32>() < percent {
                writer.write(streamline);
            }
        }
    } else if let Ok(nb) = args.get_str("--number").parse::<usize>() {
        let size = reader.header.nb_streamlines;
        let number = size.min(nb);
        if number == 0 {
            panic!(
                "You requested a subsampling of 0 streamline. \
                 Please ask for any non-zero positive number."
            );
        } else if number >= size {
            println!(
                "You requested a subsampling of {} streamlines, which is more than the total \
                 number of streamlines. The input file will simply be copied to the output file.",
                nb
            );

            reader.into_iter().for_each(|streamline| writer.write(streamline));
        } else {
            sampling_write(&mut writer, reader, number, &mut rng);
        }
    } else {
        panic!("--percent or --number can't be parsed to a positive number");
    }
}
