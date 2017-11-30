
extern crate trk_io;

use trk_io::{Affine, Point, Translation};
use trk_io::trk::Reader;

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
}
