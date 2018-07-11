// This helper module is automatically imported for all integration tests, even for those who don't
// import it. Because of this, I think it's acceptable to use a `allow(unused)` directive.
#![allow(unused)]

use tempdir::TempDir;
use trk_io::{Header, Properties, Reader, Scalars, Streamlines};

#[derive(PartialEq)]
pub struct Tract {
    pub header: Header,
    pub streamlines: Streamlines,
    pub scalars: Vec<Scalars>,
    pub properties: Vec<Properties>
}

pub fn get_random_trk_path() -> String {
    let dir = TempDir::new("trk-io").unwrap();
    let path = dir.into_path().join("out.trk");
    path.to_str().unwrap().to_string()
}

pub fn load_trk(path: &str) -> Tract {
    let mut reader = Reader::new(path).unwrap();
    let (streamlines, scalars, properties) = reader.read_all();
    Tract { header: reader.header.clone(), streamlines, scalars, properties }
}
