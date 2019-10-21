extern crate trk_io;

use trk_io::Header;

#[test]
fn test_copy_scalars_and_properties() {
    let s1 = "s1".to_string();
    let s2 = "s2".to_string();
    let p1 = "p1".to_string();
    let p2 = "p2".to_string();

    let mut header1 = Header::default();
    header1.add_scalar(&s2).unwrap();
    header1.add_property(&p2).unwrap();

    let mut header2 = Header::default();
    header2.add_scalar(&s1).unwrap();
    header2.add_scalar(&s2).unwrap();
    header2.add_property(&p1).unwrap();
    header2.add_property(&p2).unwrap();

    header2.copy_scalars_and_properties(&header1);
    assert_eq!(header2.scalars_name, vec![s2]);
    assert_eq!(header2.properties_name, vec![p2]);
}

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
