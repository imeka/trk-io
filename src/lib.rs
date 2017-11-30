
extern crate nalgebra;
extern crate byteorder;

pub mod header;
mod orientation;
pub mod streamlines;
pub mod trk;

use nalgebra::{Matrix3, Matrix4, RowVector3};

pub type Dimension = RowVector3<usize>;
pub type Point = RowVector3<f32>;
pub type Points = Vec<Point>;
pub type Affine = Matrix3<f32>;
pub type Translation = RowVector3<f32>;
type Affine4 = Matrix4<f32>;
