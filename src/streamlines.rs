
extern crate nalgebra;

use std::vec::Vec;

use streamlines::nalgebra::Matrix4;
//use streamlines::nalgebra::{
//    Dynamic, Matrix, MatrixVec, U3};

//pub type StreamlinesMatrix =
//    Matrix<f32, Dynamic, U3, MatrixVec<f32, Dynamic, U3>>;

pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

pub struct Streamlines {
    pub affine: Matrix4<f32>,
    pub lengths: Vec<usize>,
    pub offsets: Vec<usize>,
    pub data: Vec<Point>,
}

impl<'a> IntoIterator for &'a Streamlines {
    type Item = &'a [Point];
    type IntoIter = StreamlinesIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        StreamlinesIterator {
            streamlines: self,
            it_idx: 0
        }
    }
}

pub struct StreamlinesIterator<'a> {
    streamlines: &'a Streamlines,
    it_idx: usize
}

impl<'a> Iterator for StreamlinesIterator<'a> {
    type Item = &'a [Point];

    fn next(&mut self) -> Option<Self::Item> {
        if self.it_idx < self.streamlines.lengths.len() {
            let start = self.streamlines.offsets[self.it_idx];
            self.it_idx += 1;
            let end = self.streamlines.offsets[self.it_idx];

            Some(&self.streamlines.data[start..end])
        }
        else {
            None
        }
    }
}

impl Streamlines {
    pub fn new(affine: Matrix4<f32>, lengths: Vec<usize>, m: Vec<Point>) -> Streamlines {
        // CumSum over lengths
        let mut offsets = Vec::with_capacity(lengths.len());
        let mut sum = 0;
        for length in lengths.iter() {
            sum = sum + length;
            offsets.push(sum);
        }

        Streamlines {
            affine: affine,
            lengths: lengths,
            offsets: offsets,
            data: m
        }
    }
}
