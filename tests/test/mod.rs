// This helper module is automatically imported for all integration tests, even for those who don't
// import it. Because of this, I think it's acceptable to use a `allow(unused)` directive.
#![allow(unused)]

use tempdir::TempDir;
use trk_io::{Header, Reader, Streamlines};

pub fn get_random_trk_path() -> String {
    let dir = TempDir::new("trk-io").unwrap();
    let path = dir.into_path().join("out.trk");
    path.to_str().unwrap().to_string()
}

pub fn load_trk(path: &str) -> (Header, Streamlines) {
    let mut reader = Reader::new(path).unwrap();
    (reader.header.clone(), reader.read_all().0)
}
