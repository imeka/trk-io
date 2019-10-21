extern crate trk_io;

use trk_io::Header;

#[test]
fn test_add_scalar() {
    let torsion = "torsion".to_string();

    let mut header = Header::default();
    header.add_scalar(&torsion).unwrap();
    assert_eq!(header.scalars_name, vec![torsion])
}

#[test]
#[should_panic]
fn test_too_much_scalars() {
    let mut header = Header::default();
    for _ in 0..11 {
        header.add_scalar("test").unwrap();
    }
}

#[test]
#[should_panic]
fn test_scalar_name_too_long() {
    let mut header = Header::default();
    header.add_scalar("01234567890123456789a").unwrap();
}

#[test]
#[should_panic]
fn test_unicode_scalar() {
    let mut header = Header::default();
    header.add_scalar("平仮名, ひらがな").unwrap();
}

#[test]
fn test_add_property() {
    let torsion = "torsion".to_string();

    let mut header = Header::default();
    header.add_property(&torsion).unwrap();
    assert_eq!(header.properties_name, vec![torsion])
}

#[test]
#[should_panic]
fn test_too_much_properties() {
    let mut header = Header::default();
    for _ in 0..11 {
        header.add_property("test").unwrap();
    }
}

#[test]
#[should_panic]
fn test_property_name_too_long() {
    let mut header = Header::default();
    header.add_property("01234567890123456789a").unwrap();
}

#[test]
#[should_panic]
fn test_unicode_property() {
    let mut header = Header::default();
    header.add_property("平仮名, ひらがな").unwrap();
}
