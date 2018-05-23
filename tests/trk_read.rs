
extern crate trk_io;

use trk_io::{Affine, ArraySequence, Header, Point, Reader, Translation};

#[test]
fn test_load_empty() {
    let streamlines = Reader::new("data/empty.trk").read_all();
    assert_eq!(streamlines.len(), 0);
    for _ in &streamlines {
        panic!("Failed test.");
    }

    // Test generator
    for _ in Reader::new("data/empty.trk").into_iter() {
        panic!("Failed test.");
    }
}

#[test]
fn test_load_simple() {
    let first = [Point::new(0.0, 1.0, 2.0)];
    let second = [Point::new(0.0, 1.0, 2.0), Point::new(3.0, 4.0, 5.0)];
    let third = [Point::new(0.0, 1.0, 2.0),
                 Point::new(3.0, 4.0, 5.0),
                 Point::new(6.0, 7.0, 8.0),
                 Point::new(9.0, 10.0, 11.0),
                 Point::new(12.0, 13.0, 14.0)];

    let streamlines = Reader::new("data/simple.trk").read_all();
    assert_eq!(streamlines.len(), 3);
    assert_eq!(streamlines[0], first);
    assert_eq!(streamlines[1], second);
    assert_eq!(streamlines[2], third);

    // Test generator
    for (i, streamline)
            in Reader::new("data/empty.trk").into_iter().enumerate() {
        if i == 0 {
            assert_eq!(streamline, first);
        } else if i == 1 {
            assert_eq!(streamline, second);
        } else if i == 2 {
            assert_eq!(streamline, third);
        } else {
            panic!("Failed test.");
        }
    }
}

#[test]
fn test_load_standard() {
    let mut reader = Reader::new("data/standard.trk");
    let streamlines = reader.read_all();
    assert_eq!(reader.affine, Affine::new(1.0, 0.0, 0.0,
                                          0.0, 1.0, 0.0,
                                          0.0, 0.0, 1.0));
    assert_eq!(reader.translation, Translation::new(-0.5, -1.5, -1.0));

    assert_eq!(streamlines.len(), 120);
    assert_eq!(streamlines[0], [Point::new(-0.5, -1.5, 1.0),
                                Point::new(0.0, 0.0, 2.0),
                                Point::new(0.5, 1.5, 3.0)]);
    assert_eq!(streamlines[1], [Point::new(-0.5, 1.5, 1.0),
                                Point::new(0.0, 0.0, 2.0),
                                Point::new(0.5, -1.5, 3.0)]);

    // Test generator
    for streamline in Reader::new("data/empty.trk").into_iter() {
        assert_eq!(streamline.len(), 3);
    }
}

#[test]
fn test_load_standard_lps() {
    let mut reader = Reader::new("data/standard.LPS.trk");
    let streamlines = reader.read_all();
    assert_eq!(reader.affine, Affine::new(-1.0, 0.0, 0.0,
                                          0.0, -1.0, 0.0,
                                          0.0, 0.0, 1.0));
    assert_eq!(reader.translation, Translation::new(3.5, 13.5, -1.0));

    assert_eq!(streamlines.len(), 120);
    assert_eq!(streamlines[0], [Point::new(-0.5, -1.5, 1.0),
                                Point::new(0.0, 0.0, 2.0),
                                Point::new(0.5, 1.5, 3.0)]);
    assert_eq!(streamlines[1], [Point::new(-0.5, 1.5, 1.0),
                                Point::new(0.0, 0.0, 2.0),
                                Point::new(0.5, -1.5, 3.0)]);
}

#[test]
fn test_load_complex() {
    let mut reader = Reader::new("data/complex.trk");
    let streamlines = reader.read_all();
    assert_eq!(reader.affine, Affine::new(1.0, 0.0, 0.0,
                                          0.0, 1.0, 0.0,
                                          0.0, 0.0, 1.0));
    assert_eq!(reader.translation, Translation::new(-0.5, -0.5, -0.5));

    assert_eq!(streamlines.len(), 3);
    assert_eq!(streamlines[0], [Point::new(0.0, 1.0, 2.0)]);
    assert_eq!(streamlines[1], [Point::new(0.0, 1.0, 2.0),
                                Point::new(3.0, 4.0, 5.0)]);
    assert_eq!(streamlines[2], [Point::new(0.0, 1.0, 2.0),
                                Point::new(3.0, 4.0, 5.0),
                                Point::new(6.0, 7.0, 8.0),
                                Point::new(9.0, 10.0, 11.0),
                                Point::new(12.0, 13.0, 14.0)]);

    check_complex_scalars_and_properties(reader.header);
}

#[test]
fn test_load_complex_big_endian() {
    let first = [Point::new(0.0, 1.0, 2.0)];
    let second = [Point::new(0.0, 1.0, 2.0), Point::new(3.0, 4.0, 5.0)];
    let third = [Point::new(0.0, 1.0, 2.0),
                 Point::new(3.0, 4.0, 5.0),
                 Point::new(6.0, 7.0, 8.0),
                 Point::new(9.0, 10.0, 11.0),
                 Point::new(12.0, 13.0, 14.0)];

    let mut reader = Reader::new("data/complex_big_endian.trk");
    let streamlines = reader.read_all();
    assert_eq!(streamlines.len(), 3);
    assert_eq!(streamlines[0], first);
    assert_eq!(streamlines[1], second);
    assert_eq!(streamlines[2], third);
    check_complex_scalars_and_properties(reader.header);

    // Test generator
    for (i, streamline) in Reader::new(
            "data/complex_big_endian.trk").into_iter().enumerate() {
        if i == 0 {
            assert_eq!(streamline, first);
        } else if i == 1 {
            assert_eq!(streamline, second);
        } else if i == 2 {
            assert_eq!(streamline, third);
        } else {
            panic!("Failed test.");
        }
    }
}

fn check_complex_scalars_and_properties(header: Header) {
    // Scalars
    let colors_x = ArraySequence::new(
        vec![1, 2, 5], vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    let colors_y = ArraySequence::new(
        vec![1, 2, 5], vec![0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    let colors_z = ArraySequence::new(
        vec![1, 2, 5], vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
    assert!(header.scalars[0] == (String::from("colors"), colors_x));
    assert!(header.scalars[1] == (String::from("colors"), colors_y));
    assert!(header.scalars[2] == (String::from("colors"), colors_z));

    let fa = ArraySequence::new(
        vec![1, 2, 5],
        vec![0.200000003, 0.300000012, 0.400000006, 0.500000000,
             0.600000024, 0.600000024, 0.699999988, 0.800000012]);
    assert!(header.scalars[3] == (String::from("fa"), fa));

    // Properties
    assert_eq!(
        header.properties[0],
        (String::from("mean_colors"), vec![1.0, 0.0, 0.0]));
    assert_eq!(
        header.properties[1],
        (String::from("mean_colors"), vec![0.0, 1.0, 0.0]));
    assert_eq!(
        header.properties[2],
        (String::from("mean_colors"), vec![0.0, 0.0, 1.0]));
    assert_eq!(
        header.properties[3],
        (String::from("mean_curvature"),
         vec![1.11000001, 2.11000001, 3.11000001]));
    assert_eq!(
        header.properties[4],
        (String::from("mean_torsion"),
         vec![1.22000003, 2.22000003, 3.22000003]));
}
