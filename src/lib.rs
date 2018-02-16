
extern crate byteorder;
#[cfg(feature = "use_nifti")] extern crate nifti;
extern crate nalgebra;
extern crate tempdir;

pub mod affine;
mod array_sequence;
mod cheader;
mod header;
mod orientation;
mod reader;
mod writer;

use nalgebra::{Matrix3, Matrix4, RowVector3};
pub use array_sequence::ArraySequence;
pub use cheader::CHeader;
pub use header::Header;
pub use reader::Reader;
pub use writer::Writer;

pub type Dimension = RowVector3<usize>;
pub type Point = RowVector3<f32>;
pub type Points = Vec<Point>;
pub type Affine = Matrix3<f32>;
pub type Affine4 = Matrix4<f32>;
pub type Translation = RowVector3<f32>;
pub type Streamlines = ArraySequence<Point>;

pub mod tests {
    use tempdir::TempDir;
    use super::{Header, Reader, Streamlines};

    pub fn get_random_trk_path() -> String {
        let dir = TempDir::new("trk-io").unwrap();
        let path = dir.into_path().join("out.trk");
        path.to_str().unwrap().to_string()
    }

    pub fn load_trk(path: &str) -> (Header, Streamlines) {
        let mut reader = Reader::new(path);
        (reader.header.clone(), reader.read_all())
    }
}
