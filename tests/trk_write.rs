
extern crate tempdir;
extern crate trk_io;

use std::iter::FromIterator;

use tempdir::TempDir;

use trk_io::{Affine, Header, Point, Reader, Streamlines, Writer};

fn get_random_trk_path() -> String {
    let dir = TempDir::new("trk-io").unwrap();
    let path = dir.into_path().join("out.trk");
    path.to_str().unwrap().to_string()
}

fn load_trk(path: &str) -> (Header, Streamlines) {
    let mut reader = Reader::new(path);
    (reader.header.clone(), reader.read_all())
}

#[test]
fn test_write_dynamic() {
    let write_to = get_random_trk_path();
    let (original_header, original_streamlines) = load_trk("data/simple.trk");

    {
        let mut writer = Writer::new(&write_to, Some(original_header.clone()));
        writer.write_from_iter(
            [Point::new(0.0, 1.0, 2.0)].into_iter().cloned(), 1);

        let v = vec![Point::new(0.0, 1.0, 2.0), Point::new(3.0, 4.0, 5.0)];
        writer.write_from_iter(v, 2);

        let v = Vec::from_iter((0..15));
        let iter = v.chunks(3).map(
            |ints| Point::new(ints[0] as f32, ints[1] as f32, ints[2] as f32)
        );
        writer.write_from_iter(iter, 5);
    }

    let (written_header, written_streamlines) = load_trk(&write_to);
    assert!(original_header == written_header);
    assert!(original_streamlines == written_streamlines);
}

#[test]
fn test_write_empty() {
    let write_to = get_random_trk_path();
    let (original_header, original_streamlines) = load_trk("data/empty.trk");

    {
        let mut writer = Writer::new(&write_to, Some(original_header.clone()));
        writer.write_all(&Streamlines::new(vec![], vec![]));
    }

    let (written_header, written_streamlines) = load_trk(&write_to);
    assert!(original_header == written_header);
    assert!(original_streamlines == written_streamlines);
}

#[test]
fn test_write_simple() {
    let write_to = get_random_trk_path();
    let (original_header, original_streamlines) = load_trk("data/simple.trk");

    {
        let mut writer = Writer::new(&write_to, Some(original_header.clone()));
        writer.write_all(&original_streamlines);
    }

    let (written_header, written_streamlines) = load_trk(&write_to);
    assert!(original_header == written_header);
    assert!(original_streamlines == written_streamlines);
}

#[test]
fn test_write_standard() {
    let write_to = get_random_trk_path();
    let (original_header, original_streamlines) =
        load_trk("data/standard.trk");

    {
        let mut writer = Writer::new(&write_to, Some(original_header.clone()));
        writer.write(&original_streamlines[0]);
        writer.write(&original_streamlines[1]);
        writer.write(&original_streamlines[2]);
    }

    let (written_header, written_streamlines) = load_trk(&write_to);
    assert_eq!(written_header.nb_streamlines, 3);
    assert_eq!(written_streamlines[0], [Point::new(-0.5, -1.5, 1.0),
                                        Point::new(0.0, 0.0, 2.0),
                                        Point::new(0.5, 1.5, 3.0)]);
}

#[test]
fn test_write_standard_lps() {
    let write_to = get_random_trk_path();
    let (original_header, original_streamlines) =
        load_trk("data/standard.LPS.trk");

    {
        let mut writer = Writer::new(&write_to, Some(original_header.clone()));
        for i in 0..10 {
            writer.write(&original_streamlines[i]);
        }
    }

    let (written_header, written_streamlines) = load_trk(&write_to);
    assert_eq!(written_header.nb_streamlines, 10);
    assert_eq!(written_header.affine, Affine::new(-1.0, 0.0, 0.0,
                                                  0.0, -1.0, 0.0,
                                                  0.0, 0.0, 1.0));
    assert_eq!(written_streamlines[0], [Point::new(-0.5, -1.5, 1.0),
                                        Point::new(0.0, 0.0, 2.0),
                                        Point::new(0.5, 1.5, 3.0)]);
}

#[test]
fn test_write_complex() {
    // TODO This is not currently testing anything interesting because we
    // remove scalars and properties information before saving. This test needs
    // to be updated when we handle scalars and properties.
    let write_to = get_random_trk_path();
    let (original_header, original_streamlines) = load_trk("data/complex.trk");

    {
        let mut writer = Writer::new(&write_to, Some(original_header.clone()));
        writer.write_all(&original_streamlines);
    }

    let (written_header, written_streamlines) = load_trk(&write_to);
    assert_eq!(written_header.nb_streamlines, 3);
    assert!(original_streamlines == written_streamlines);
}
