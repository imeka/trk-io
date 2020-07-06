pub mod affine;
mod array_sequence;
mod cheader;
mod header;
pub mod orientation;
mod reader;
mod tractogram;
mod writer;

use byteorder::LittleEndian;
use nalgebra::{Matrix3, Matrix4, Vector3};

pub use array_sequence::ArraySequence;
pub use cheader::CHeader;
pub use header::Header;
pub use reader::Reader;
pub use tractogram::{Point, Points, Streamlines, Tractogram, TractogramItem};
pub use writer::Writer;

pub type Affine = Matrix3<f32>;
pub type Affine4 = Matrix4<f32>;
pub type Translation = Vector3<f32>;

/// trk-io will always write trk in LE, but it wll also try BE when reading
type TrkEndianness = LittleEndian;
