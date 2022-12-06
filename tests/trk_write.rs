mod test;

use std::iter::FromIterator;

use anyhow::Result;

use test::{get_random_trk_path, load_trk};
use trk_io::{Affine4, Point, Reader, Writer};

// write(Tractogram) is tested in write_empty and write_simple.
// write(TractogramItem) is tested in test_write_tractogram_item_simple and write_complex.
// write(RefTractogramItem) is tested in write_ref_tractogram_item.
// write(&[Point]) is tested in write_standard and write_standard_lps.
// write_from_iter is tested in write_dynamic.

#[test]
fn test_write_dynamic() -> Result<()> {
    let write_to = get_random_trk_path();
    let (original_header, original_tractogram) = load_trk("data/simple.trk");

    // This seemingly useless { scope } is *required* because Writer::drop must be called
    {
        let mut writer = Writer::new(&write_to, Some(original_header.clone()))?;
        writer.write_from_iter([Point::new(0.0, 1.0, 2.0)].iter().cloned(), 1);

        let v = vec![Point::new(0.0, 1.0, 2.0), Point::new(3.0, 4.0, 5.0)];
        writer.write_from_iter(v, 2);

        let v = Vec::from_iter(0..15);
        let iter =
            v.chunks(3).map(|ints| Point::new(ints[0] as f32, ints[1] as f32, ints[2] as f32));
        writer.write_from_iter(iter, 5);
    }

    assert!((original_header, original_tractogram) == load_trk(&write_to));
    Ok(())
}

#[test]
fn test_write_empty() -> Result<()> {
    let write_to = get_random_trk_path();
    let (original_header, original_tractogram) = load_trk("data/empty.trk");

    {
        let mut writer = Writer::new(&write_to, Some(original_header.clone()))?;
        writer.write(original_tractogram.clone());
    }

    assert!((original_header, original_tractogram) == load_trk(&write_to));
    Ok(())
}

#[test]
fn test_write_simple() -> Result<()> {
    let write_to = get_random_trk_path();
    let (original_header, original_tractogram) = load_trk("data/simple.trk");

    {
        let mut writer = Writer::new(&write_to, Some(original_header.clone()))?;
        writer.write(original_tractogram.clone());
    }

    assert!((original_header, original_tractogram) == load_trk(&write_to));
    Ok(())
}

#[test]
fn test_write_points_simple() -> Result<()> {
    let write_to = get_random_trk_path();
    let (original_header, original_tractogram) = load_trk("data/simple.trk");

    {
        let mut writer = Writer::new(&write_to, Some(original_header.clone()))?;
        for streamline in original_tractogram.streamlines.into_iter() {
            writer.write(streamline);
        }
    }

    assert!((original_header, original_tractogram) == load_trk(&write_to));
    Ok(())
}

#[test]
fn test_write_tractogram_item_simple() -> Result<()> {
    let write_to = get_random_trk_path();
    let reader = Reader::new("data/simple.trk")?;

    {
        let mut writer = Writer::new(&write_to, Some(reader.header.clone()))?;
        for item in reader.into_iter() {
            writer.write(item);
        }
    }

    let (original_header, original_tractogram) = load_trk("data/simple.trk");
    assert!((original_header, original_tractogram) == load_trk(&write_to));
    Ok(())
}

#[test]
fn test_write_ref_tractogram_item_simple() -> Result<()> {
    let write_to = get_random_trk_path();
    let (original_header, original_tractogram) = load_trk("data/simple.trk");

    {
        let mut writer = Writer::new(&write_to, Some(original_header.clone()))?;
        for ref_item in original_tractogram.into_iter() {
            writer.write(ref_item);
        }
    }

    assert!((original_header, original_tractogram) == load_trk(&write_to));
    Ok(())
}

#[test]
fn test_write_standard() -> Result<()> {
    let write_to = get_random_trk_path();
    let (original_header, original_tractogram) = load_trk("data/standard.trk");

    {
        let mut writer = Writer::new(&write_to, Some(original_header))?;
        writer.write(&original_tractogram.streamlines[0]);
        writer.write(&original_tractogram.streamlines[1]);
        writer.write(&original_tractogram.streamlines[2]);
    }

    let (header, tractogram) = load_trk(&write_to);
    assert_eq!(header.nb_streamlines, 3);
    assert_eq!(
        tractogram.streamlines[0],
        [Point::new(-0.5, -1.5, 1.0), Point::new(0.0, 0.0, 2.0), Point::new(0.5, 1.5, 3.0)]
    );
    Ok(())
}

#[test]
fn test_write_standard_lps() -> Result<()> {
    let write_to = get_random_trk_path();
    let (original_header, original_tractogram) = load_trk("data/standard.LPS.trk");

    {
        let mut writer = Writer::new(&write_to, Some(original_header.clone()))?;
        #[rustfmt::skip]
        assert_eq!(
            writer.affine4,
            Affine4::new(
                -1.0, 0.0, 0.0, 3.5,
                0.0, -1.0, 0.0, 13.5,
                0.0, 0.0, 1.0, 1.0,
                0.0, 0.0, 0.0, 1.0,
            )
        );
        for i in 0..10 {
            writer.write(&original_tractogram.streamlines[i]);
        }
    }

    let (header, tractogram) = load_trk(&write_to);
    assert_eq!(header.nb_streamlines, 10);
    #[rustfmt::skip]
    assert_eq!(
        header.affine4_to_rasmm,
        Affine4::new(
            -1.0, 0.0, 0.0, 3.5,
            0.0, -1.0, 0.0, 13.5,
            0.0, 0.0, 1.0, -1.0,
            0.0, 0.0, 0.0, 1.0,
        )
    );
    assert_eq!(
        tractogram.streamlines[0],
        [Point::new(-0.5, -1.5, 1.0), Point::new(0.0, 0.0, 2.0), Point::new(0.5, 1.5, 3.0)]
    );
    Ok(())
}

#[test]
fn test_write_complex() -> Result<()> {
    let write_to = get_random_trk_path();
    let reader = Reader::new("data/complex.trk")?;

    {
        let mut writer = Writer::new(&write_to, Some(reader.header.clone()))?;
        for item in reader.into_iter() {
            writer.write(item);
        }
    }

    let (original_header, original_tractogram) = load_trk("data/complex.trk");
    assert!((original_header, original_tractogram) == load_trk(&write_to));
    Ok(())
}
