use anyhow::Result;
use docopt::Docopt;
use trk_io::{Header, Point, Reader, Writer};

static USAGE: &'static str = "
Color a TrackVis (.trk) file.

This will add 3 scalars (color_x, color_y, color_z) per point. Please note that coloring by 'local'
orientation may be useless as some programs already use this method by default to color the
streamlines.

The valid range for the `uniform` option is [0, 255], thus red is `255, 0, 0`.

Usage:
  trk_color uniform <r> <g> <b> <input> <output> [options]
  trk_color local <input> <output> [options]
  trk_color (-h | --help)
  trk_color (-v | --version)

Options:
  -h --help      Show this screen.
  -v --version   Show version.
";

fn main() -> Result<()> {
    let version = String::from(env!("CARGO_PKG_VERSION"));
    let args = Docopt::new(USAGE)
        .and_then(|dopt| dopt.version(Some(version)).parse())
        .unwrap_or_else(|e| e.exit());

    let reader = Reader::new(args.get_str("<input>"))?;
    let mut header = reader.header.clone();
    header.add_scalar("color_x")?;
    header.add_scalar("color_y")?;
    header.add_scalar("color_z")?;

    if args.get_bool("uniform") {
        let r = args.get_str("<r>").parse::<u32>()?;
        let g = args.get_str("<g>").parse::<u32>()?;
        let b = args.get_str("<b>").parse::<u32>()?;
        uniform(reader, header, args.get_str("<output>"), r, g, b);
    } else if args.get_bool("local") {
        local(reader, header, args.get_str("<output>"));
    }

    Ok(())
}

fn uniform(reader: Reader, header: Header, write_to: &str, r: u32, g: u32, b: u32) {
    let (r, g, b) = (r as f32, g as f32, b as f32);
    let mut writer = Writer::new(write_to, Some(header)).unwrap();
    for (streamline, mut scalars, properties) in reader.into_iter() {
        for _ in 0..streamline.len() {
            scalars.push(r);
            scalars.push(g);
            scalars.push(b);
        }

        writer.write((streamline, scalars, properties));
    }
}

fn local(reader: Reader, header: Header, write_to: &str) {
    let mut writer = Writer::new(write_to, Some(header)).unwrap();
    for (streamline, mut scalars, properties) in reader.into_iter() {
        let mut add = |p1: &Point, p2: &Point| {
            let x = p2.x - p1.x;
            let y = p2.y - p1.y;
            let z = p2.z - p1.z;
            let norm = (x.powi(2) + y.powi(2) + z.powi(2)).sqrt();
            scalars.push((x / norm).abs() * 255.0);
            scalars.push((y / norm).abs() * 255.0);
            scalars.push((z / norm).abs() * 255.0);
        };

        // Manage first point
        add(&streamline[0], &streamline[1]);

        for p in streamline.windows(3) {
            add(&p[0], &p[2]);
        }

        // Manage last point
        add(&streamline[streamline.len() - 2], &streamline[streamline.len() - 1]);

        writer.write((streamline, scalars, properties));
    }
}
