extern crate tempdir;
extern crate trk_io;

mod test;

use std::iter::FromIterator;

use trk_io::{Affine4, Point, Streamlines, Writer};
use test::{get_random_trk_path, load_trk};

#[test]
fn test_write_dynamic() {
    let write_to = get_random_trk_path();
    let original = load_trk("data/simple.trk");

    {
        let mut writer = Writer::new(
            &write_to, Some(original.header.clone())).unwrap();
        writer.write_from_iter(
            [Point::new(0.0, 1.0, 2.0)].iter().cloned(), 1);

        let v = vec![Point::new(0.0, 1.0, 2.0), Point::new(3.0, 4.0, 5.0)];
        writer.write_from_iter(v, 2);

        let v = Vec::from_iter(0..15);
        let iter = v.chunks(3).map(
            |ints| Point::new(ints[0] as f32, ints[1] as f32, ints[2] as f32)
        );
        writer.write_from_iter(iter, 5);
    }

    assert!(original == load_trk(&write_to));
}

#[test]
fn test_write_empty() {
    let write_to = get_random_trk_path();
    let original = load_trk("data/empty.trk");

    {
        let mut writer = Writer::new(
            &write_to, Some(original.header.clone())).unwrap();
        writer.write_all(
            &Streamlines::new(vec![], vec![]),
            original.scalars.clone(),
            original.properties.clone());
    }

    assert!(original == load_trk(&write_to));
}

#[test]
fn test_write_simple() {
    let write_to = get_random_trk_path();
    let original = load_trk("data/simple.trk");

    {
        let mut writer = Writer::new(
            &write_to, Some(original.header.clone())).unwrap();
        writer.write_all(
            &original.streamlines,
            original.scalars.clone(),
            original.properties.clone());
    }

    assert!(original == load_trk(&write_to));
}

#[test]
fn test_write_standard() {
    let write_to = get_random_trk_path();
    let original = load_trk("data/standard.trk");

    {
        let mut writer = Writer::new(
            &write_to, Some(original.header.clone())).unwrap();
        writer.write(&original.streamlines[0]);
        writer.write(&original.streamlines[1]);
        writer.write(&original.streamlines[2]);
    }

    let written = load_trk(&write_to);
    assert_eq!(written.header.nb_streamlines, 3);
    assert_eq!(written.streamlines[0], [Point::new(-0.5, -1.5, 1.0),
                                        Point::new(0.0, 0.0, 2.0),
                                        Point::new(0.5, 1.5, 3.0)]);
}

#[test]
fn test_write_standard_lps() {
    let write_to = get_random_trk_path();
    let original = load_trk("data/standard.LPS.trk");

    {
        let mut writer = Writer::new(
            &write_to, Some(original.header.clone())).unwrap();
        assert_eq!(writer.affine4, Affine4::new(-1.0, 0.0, 0.0, 3.5,
                                                0.0, -1.0, 0.0, 13.5,
                                                0.0, 0.0, 1.0, 1.0,
                                                0.0, 0.0, 0.0, 1.0));
        for i in 0..10 {
            writer.write(&original.streamlines[i]);
        }
    }

    let written = load_trk(&write_to);
    assert_eq!(written.header.nb_streamlines, 10);
    assert_eq!(written.header.affine4, Affine4::new(-1.0, 0.0, 0.0, 3.5,
                                                    0.0, -1.0, 0.0, 13.5,
                                                    0.0, 0.0, 1.0, -1.0,
                                                    0.0, 0.0, 0.0, 1.0));
    assert_eq!(written.streamlines[0], [Point::new(-0.5, -1.5, 1.0),
                                        Point::new(0.0, 0.0, 2.0),
                                        Point::new(0.5, 1.5, 3.0)]);
}

#[test]
fn test_write_complex() {
    let write_to = get_random_trk_path();
    let original = load_trk("data/complex.trk");

    {
        let mut writer = Writer::new(
            &write_to, Some(original.header.clone())).unwrap();
        writer.write_all(
            &original.streamlines,
            original.scalars.clone(),
            original.properties.clone());
    }

    assert!(original == load_trk(&write_to));
}
