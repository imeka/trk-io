extern crate tempdir;
extern crate trk_io;

mod test;

use std::iter::FromIterator;

use trk_io::{Affine4, Point, TractogramItem, Writer};
use test::{get_random_trk_path, load_trk};

#[test]
fn test_write_dynamic() {
    let write_to = get_random_trk_path();
    let (original_header, original_tractogram) = load_trk("data/simple.trk");

    // This seemingly useless { scope } is *required* because Writer::drop must be called
    {
        let mut writer = Writer::new(&write_to, Some(original_header.clone())).unwrap();
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

    assert!((original_header, original_tractogram) == load_trk(&write_to));
}

#[test]
fn test_write_empty() {
    let write_to = get_random_trk_path();
    let (original_header, original_tractogram) = load_trk("data/empty.trk");

    {
        let mut writer = Writer::new(&write_to, Some(original_header.clone())).unwrap();
        writer.write_all(original_tractogram.clone());
    }

    assert!((original_header, original_tractogram) == load_trk(&write_to));
}

#[test]
fn test_write_simple() {
    let write_to = get_random_trk_path();
    let (original_header, original_tractogram) = load_trk("data/simple.trk");

    {
        let mut writer = Writer::new(&write_to, Some(original_header.clone())).unwrap();
        writer.write_all(original_tractogram.clone());
    }

    assert!((original_header, original_tractogram) == load_trk(&write_to));
}

#[test]
fn test_write_points_simple() {
    let write_to = get_random_trk_path();
    let (original_header, original_tractogram) = load_trk("data/simple.trk");

    {
        let mut writer = Writer::new(&write_to, Some(original_header.clone())).unwrap();
        for streamline in original_tractogram.streamlines.into_iter() {
            writer.write_points(streamline.to_vec());
        }
    }

    assert!((original_header, original_tractogram) == load_trk(&write_to));
}

#[test]
fn test_write_standard() {
    let write_to = get_random_trk_path();
    let (original_header, original_tractogram) = load_trk("data/standard.trk");

    {
        let mut writer = Writer::new(&write_to, Some(original_header)).unwrap();
        writer.write(TractogramItem::from_slice(&original_tractogram.streamlines[0]));
        writer.write(TractogramItem::from_slice(&original_tractogram.streamlines[1]));
        writer.write(TractogramItem::from_slice(&original_tractogram.streamlines[2]));
    }

    let (header, tractogram) = load_trk(&write_to);
    assert_eq!(header.nb_streamlines, 3);
    assert_eq!(tractogram.streamlines[0], [Point::new(-0.5, -1.5, 1.0),
                                           Point::new(0.0, 0.0, 2.0),
                                           Point::new(0.5, 1.5, 3.0)]);
}

#[test]
fn test_write_standard_lps() {
    let write_to = get_random_trk_path();
    let (original_header, original_tractogram) = load_trk("data/standard.LPS.trk");

    {
        let mut writer = Writer::new(&write_to, Some(original_header.clone())).unwrap();
        assert_eq!(writer.affine4, Affine4::new(-1.0, 0.0, 0.0, 3.5,
                                                0.0, -1.0, 0.0, 13.5,
                                                0.0, 0.0, 1.0, 1.0,
                                                0.0, 0.0, 0.0, 1.0));
        for i in 0..10 {
            writer.write(TractogramItem::from_slice(&original_tractogram.streamlines[i]));
        }
    }

    let (header, tractogram) = load_trk(&write_to);
    assert_eq!(header.nb_streamlines, 10);
    assert_eq!(header.affine4, Affine4::new(-1.0, 0.0, 0.0, 3.5,
                                            0.0, -1.0, 0.0, 13.5,
                                            0.0, 0.0, 1.0, -1.0,
                                            0.0, 0.0, 0.0, 1.0));
    assert_eq!(tractogram.streamlines[0], [Point::new(-0.5, -1.5, 1.0),
                                           Point::new(0.0, 0.0, 2.0),
                                           Point::new(0.5, 1.5, 3.0)]);
}

#[test]
fn test_write_complex() {
    let write_to = get_random_trk_path();
    let (original_header, original_tractogram) = load_trk("data/complex.trk");

    {
        let mut writer = Writer::new(&write_to, Some(original_header.clone())).unwrap();
        writer.write_all(original_tractogram.clone());
    }

    assert!((original_header, original_tractogram) == load_trk(&write_to));
}
