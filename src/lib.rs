extern crate byteorder;
#[cfg(feature = "use_nifti")] extern crate nifti;
extern crate nalgebra;

pub mod affine;
#[cfg(feature = "use_nifti")] pub mod affine_nifti;
mod array_sequence;
mod cheader;
mod header;
pub mod orientation;
mod tractogram;
mod reader;
mod writer;

use byteorder::LittleEndian;
use nalgebra::{Matrix3, Matrix4, Vector3};

pub use array_sequence::ArraySequence;
pub use cheader::CHeader;
pub use header::Header;
pub use tractogram::{Point, Points, Streamlines, Tractogram, TractogramItem};
pub use reader::Reader;
pub use writer::Writer;

pub type Affine = Matrix3<f32>;
pub type Affine4 = Matrix4<f32>;
pub type Translation = Vector3<f32>;

/// trk-io will always write trk in LE, but it wll also try BE when reading
type TrkEndianness = LittleEndian;
