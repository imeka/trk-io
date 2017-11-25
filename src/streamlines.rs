
use std::vec::Vec;

use nalgebra::{Matrix4, Vector3};

pub type Point = Vector3<f32>;
pub type Affine = Matrix4<f32>;

pub struct Streamlines {
    pub affine: Affine,
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
    pub fn new(affine: Affine, lengths: Vec<usize>, m: Vec<Point>) -> Streamlines {
        // CumSum over lengths
        let mut offsets = Vec::with_capacity(lengths.len() + 1);
        let mut sum = 0;
        for length in &lengths {
            offsets.push(sum);
            sum = sum + length;
        }
        offsets.push(sum);

        Streamlines {
            affine: affine,
            lengths: lengths,
            offsets: offsets,
            data: m
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iterator() {
        let streamlines = Streamlines::new(
            Affine::identity(),
            vec![2, 3],
            vec![Point::new(1.0, 0.0, 0.0),
                 Point::new(2.0, 0.0, 0.0),
                 Point::new(0.0, 1.0, 0.0),
                 Point::new(0.0, 2.0, 0.0),
                 Point::new(0.0, 3.0, 0.0)]);
        let mut iter = streamlines.into_iter();
        assert_eq!(iter.next(),
                   Some(vec![Point::new(1.0, 0.0, 0.0),
                             Point::new(2.0, 0.0, 0.0)].as_slice()));
        assert_eq!(iter.next(),
                   Some(vec![Point::new(0.0, 1.0, 0.0),
                             Point::new(0.0, 2.0, 0.0),
                             Point::new(0.0, 3.0, 0.0)].as_slice()));
        assert_eq!(iter.next(), None);
    }
}
