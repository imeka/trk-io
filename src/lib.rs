
extern crate byteorder;
extern crate nalgebra;

mod cheader;
mod header;
mod orientation;
mod reader;
mod streamlines;
mod writer;

use nalgebra::{Matrix3, Matrix4, RowVector3};
pub use cheader::CHeader;
pub use header::Header;
pub use reader::Reader;
pub use streamlines::Streamlines;
pub use writer::Writer;

pub type Dimension = RowVector3<usize>;
pub type Point = RowVector3<f32>;
pub type Points = Vec<Point>;
pub type Affine = Matrix3<f32>;
pub type Translation = RowVector3<f32>;
type Affine4 = Matrix4<f32>;
