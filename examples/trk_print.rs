use anyhow::Result;
use docopt::Docopt;

use trk_io::{Point, Reader};

static USAGE: &'static str = "
Print the first points of the first streamlines of a trk file.

Usage:
  trk_print <input> first <nb> [options]
  trk_print <input> at <idx> [options]
  trk_print (-h | --help)
  trk_print (-v | --version)

Options:
  -p --precision=<n>   Print `n` decimals. [default: 2]
  -u --upto=<n>        Print up to `n` points per streamline. Will take the first `n / 2` and the
                       last `n / 2` points. An odd number is ignored in favor of `n - 1`.
                       Print all points if unspecified.
  -h --help            Show this screen.
  -v --version         Show version.
";

fn main() -> Result<()> {
    let version = String::from(env!("CARGO_PKG_VERSION"));
    let args = Docopt::new(USAGE)
        .and_then(|dopt| dopt.version(Some(version)).parse())
        .unwrap_or_else(|e| e.exit());

    let precision = args.get_str("--precision").parse::<usize>()?;
    let print = |p: &Point| {
        println!("({:.*} {:.*} {:.*})", precision, p[0], precision, p[1], precision, p[2]);
    };

    let upto = args.get_str("--upto").parse::<usize>().unwrap_or(std::usize::MAX);
    let first_part = upto / 2;

    let reader = Reader::new(args.get_str("<input>"))?.into_streamlines_iter();
    if args.get_bool("first") {
        // nb - 1 because we don't want to print the last \n
        let nb = args.get_str("<nb>").parse::<usize>()? - 1;

        for (i, streamline) in reader.into_iter().enumerate() {
            let len = streamline.len();
            if len > upto {
                streamline[0..first_part].iter().for_each(&print);
                println!("...");

                let second_part = len - first_part;
                streamline[second_part..].iter().for_each(&print);
            } else {
                streamline.iter().for_each(&print)
            }

            if i == nb {
                break;
            }
            println!("");
        }
    } else {
        let idx = args.get_str("<idx>").parse::<usize>()?;
        for (i, streamline) in reader.into_iter().enumerate() {
            if i == idx {
                let len = streamline.len();
                if len > upto {
                    streamline[0..first_part].iter().for_each(&print);
                    println!("...");

                    let second_part = len - first_part;
                    streamline[second_part..].iter().for_each(&print);
                } else {
                    streamline.iter().for_each(&print)
                }
                break;
            }
        }
    }

    Ok(())
}
