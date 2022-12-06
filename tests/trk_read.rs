use anyhow::Result;
use nalgebra::Vector3;

use trk_io::{Affine, ArraySequence, Header, Point, Reader, Tractogram, Translation};

#[test]
fn test_load_empty() -> Result<()> {
    let Tractogram { streamlines, scalars, properties } =
        Reader::new("data/empty.trk")?.tractogram();

    assert_eq!(streamlines.len(), 0);
    assert!(scalars.is_empty());
    assert!(properties.is_empty());

    // Test generator
    let reader = Reader::new("data/empty.trk")?;
    assert_eq!(reader.into_iter().count(), 0);

    Ok(())
}

#[test]
fn test_load_simple() -> Result<()> {
    let first = [Point::new(0.0, 1.0, 2.0)];
    let second = [Point::new(0.0, 1.0, 2.0), Point::new(3.0, 4.0, 5.0)];
    let third = [
        Point::new(0.0, 1.0, 2.0),
        Point::new(3.0, 4.0, 5.0),
        Point::new(6.0, 7.0, 8.0),
        Point::new(9.0, 10.0, 11.0),
        Point::new(12.0, 13.0, 14.0),
    ];

    // Test the complete tractogram reading
    let Tractogram { streamlines, scalars, properties } =
        Reader::new("data/simple.trk")?.tractogram();
    assert_eq!(streamlines.len(), 3);
    assert_eq!(streamlines[0], first);
    assert_eq!(streamlines[1], second);
    assert_eq!(streamlines[2], third);
    assert!(scalars.is_empty());
    assert!(properties.is_empty());

    // Test the complete points reading
    let streamlines = Reader::new("data/simple.trk")?.streamlines();
    assert_eq!(streamlines.len(), 3);
    assert_eq!(streamlines[0], first);
    assert_eq!(streamlines[1], second);
    assert_eq!(streamlines[2], third);

    // Test the tractogram items generator
    let reader = Reader::new("data/simple.trk")?;
    for (i, (streamline, _, _)) in reader.into_iter().enumerate() {
        match i {
            0 => assert_eq!(streamline, first),
            1 => assert_eq!(streamline, second),
            2 => assert_eq!(streamline, third),
            _ => panic!("Failed test."),
        }
    }

    // Test the streamlines generator
    let reader = Reader::new("data/simple.trk")?;
    for (i, streamline) in reader.into_streamlines_iter().enumerate() {
        match i {
            0 => assert_eq!(streamline, first),
            1 => assert_eq!(streamline, second),
            2 => assert_eq!(streamline, third),
            _ => panic!("Failed test."),
        }
    }

    Ok(())
}

#[test]
fn test_load_standard() -> Result<()> {
    let mut reader = Reader::new("data/standard.trk")?;
    let Tractogram { streamlines, scalars, properties } = reader.tractogram();

    assert_eq!(reader.affine_to_rasmm, Affine::identity());
    assert_eq!(reader.translation, Translation::new(-0.5, -1.5, -1.0));

    assert_eq!(streamlines.len(), 120);
    assert_eq!(
        streamlines[0],
        [Point::new(-0.5, -1.5, 1.0), Point::new(0.0, 0.0, 2.0), Point::new(0.5, 1.5, 3.0)]
    );
    assert_eq!(
        streamlines[1],
        [Point::new(-0.5, 1.5, 1.0), Point::new(0.0, 0.0, 2.0), Point::new(0.5, -1.5, 3.0)]
    );
    assert!(scalars.is_empty());
    assert!(properties.is_empty());

    // Test generator
    for (streamline, _, _) in Reader::new("data/standard.trk")? {
        assert_eq!(streamline.len(), 3);
    }

    Ok(())
}

#[test]
fn test_load_standard_lps() -> Result<()> {
    let mut reader = Reader::new("data/standard.LPS.trk")?;
    let Tractogram { streamlines, scalars, properties } = reader.tractogram();
    assert_eq!(reader.affine_to_rasmm, Affine::from_diagonal(&Vector3::new(-1.0, -1.0, 1.0)));
    assert_eq!(reader.translation, Translation::new(3.5, 13.5, -1.0));

    assert_eq!(streamlines.len(), 120);
    assert_eq!(
        streamlines[0],
        [Point::new(-0.5, -1.5, 1.0), Point::new(0.0, 0.0, 2.0), Point::new(0.5, 1.5, 3.0)]
    );
    assert_eq!(
        streamlines[1],
        [Point::new(-0.5, 1.5, 1.0), Point::new(0.0, 0.0, 2.0), Point::new(0.5, -1.5, 3.0)]
    );
    assert!(scalars.is_empty());
    assert!(properties.is_empty());

    Ok(())
}

#[test]
fn test_load_complex() -> Result<()> {
    let mut reader = Reader::new("data/complex.trk")?;
    let Tractogram { streamlines, scalars, properties } = reader.tractogram();
    assert_eq!(reader.affine_to_rasmm, Affine::identity());
    assert_eq!(reader.translation, Translation::new(-0.5, -0.5, -0.5));

    assert_eq!(streamlines.len(), 3);
    assert_eq!(streamlines[0], [Point::new(0.0, 1.0, 2.0)]);
    assert_eq!(streamlines[1], [Point::new(0.0, 1.0, 2.0), Point::new(3.0, 4.0, 5.0)]);
    assert_eq!(
        streamlines[2],
        [
            Point::new(0.0, 1.0, 2.0),
            Point::new(3.0, 4.0, 5.0),
            Point::new(6.0, 7.0, 8.0),
            Point::new(9.0, 10.0, 11.0),
            Point::new(12.0, 13.0, 14.0)
        ]
    );

    check_complex_scalars_and_properties(reader.header, scalars, properties);

    Ok(())
}

#[test]
fn test_load_complex_big_endian() -> Result<()> {
    let first = [Point::new(0.0, 1.0, 2.0)];
    let second = [Point::new(0.0, 1.0, 2.0), Point::new(3.0, 4.0, 5.0)];
    let third = [
        Point::new(0.0, 1.0, 2.0),
        Point::new(3.0, 4.0, 5.0),
        Point::new(6.0, 7.0, 8.0),
        Point::new(9.0, 10.0, 11.0),
        Point::new(12.0, 13.0, 14.0),
    ];

    let mut reader = Reader::new("data/complex_big_endian.trk")?;
    let Tractogram { streamlines, scalars, properties } = reader.tractogram();
    assert_eq!(streamlines.len(), 3);
    assert_eq!(streamlines[0], first);
    assert_eq!(streamlines[1], second);
    assert_eq!(streamlines[2], third);
    check_complex_scalars_and_properties(reader.header, scalars, properties);

    // Test generator
    let reader = Reader::new("data/complex_big_endian.trk")?;
    for (i, streamline) in reader.into_streamlines_iter().enumerate() {
        match i {
            0 => assert_eq!(streamline, first),
            1 => assert_eq!(streamline, second),
            2 => assert_eq!(streamline, third),
            _ => panic!("Failed test."),
        }
    }

    Ok(())
}

fn check_complex_scalars_and_properties(
    header: Header,
    scalars: ArraySequence<f32>,
    properties: ArraySequence<f32>,
) {
    // Scalars
    assert_eq!(
        header.scalars_name,
        vec![
            String::from("colors"),
            String::from("colors"),
            String::from("colors"),
            String::from("fa")
        ]
    );
    assert_eq!(&scalars[0], &[1.0, 0.0, 0.0, 0.200000003]);
    assert_eq!(&scalars[1], &[0.0, 1.0, 0.0, 0.300000012, 0.0, 1.0, 0.0, 0.400000006]);
    #[rustfmt::skip]
    assert_eq!(
        &scalars[2],
        &[
            0.0, 0.0, 1.0, 0.500000000,
            0.0, 0.0, 1.0, 0.600000024,
            0.0, 0.0, 1.0, 0.600000024,
            0.0, 0.0, 1.0, 0.699999988,
            0.0, 0.0, 1.0, 0.800000012
        ]
    );

    // Properties
    assert_eq!(
        header.properties_name,
        vec![
            String::from("mean_colors"),
            String::from("mean_colors"),
            String::from("mean_colors"),
            String::from("mean_curvature"),
            String::from("mean_torsion")
        ]
    );
    assert_eq!(&properties[0], &[1.0, 0.0, 0.0, 1.11000001, 1.22000003]);
    assert_eq!(&properties[1], &[0.0, 1.0, 0.0, 2.11000001, 2.22000003]);
    assert_eq!(&properties[2], &[0.0, 0.0, 1.0, 3.11000001, 3.22000003]);
}
