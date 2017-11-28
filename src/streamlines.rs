
use std::ops::Index;
use std::vec::Vec;

use nalgebra::{RowVector3};

use {Affine, Translation};

pub type Point = RowVector3<f32>;

pub struct Streamlines {
    pub affine: Affine,
    pub translation: Translation,
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
            self.it_idx += 1;
            Some(&self.streamlines[self.it_idx - 1])
        }
        else {
            None
        }
    }
}

impl Index<usize> for Streamlines {
    type Output = [Point];

    fn index<'a>(&'a self, i: usize) -> &'a Self::Output {
        let start = self.offsets[i];
        let end = self.offsets[i + 1];
        &self.data[start..end]
    }
}

impl Streamlines {
    pub fn new(
        affine: Affine,
        translation: Translation,
        lengths: Vec<usize>,
        m: Vec<Point>
    ) -> Streamlines {
        // CumSum over lengths
        let mut offsets = Vec::with_capacity(lengths.len() + 1);
        let mut sum = 0;
        for length in &lengths {
            offsets.push(sum);
            sum = sum + length;
        }
        offsets.push(sum);

        Streamlines { affine, translation, lengths, offsets, data: m }
    }

    pub fn len(&self) -> usize {
        self.lengths.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construction() {
        let streamlines = Streamlines::new(
            Affine::identity(),
            Translation::new(0.0, 0.0, 0.0),
            vec![2, 3, 2],
            vec![Point::new(1.0, 0.0, 0.0),
                 Point::new(2.0, 0.0, 0.0),
                 Point::new(0.0, 1.0, 0.0),
                 Point::new(0.0, 2.0, 0.0),
                 Point::new(0.0, 3.0, 0.0),
                 Point::new(0.0, 0.0, 1.0),
                 Point::new(0.0, 0.0, 2.0)]);
        assert_eq!(streamlines.len(), 3);
        assert_eq!(streamlines.offsets, vec![0, 2, 5, 7]);
    }

    #[test]
    fn test_iterator() {
        let streamlines = Streamlines::new(
            Affine::identity(),
            Translation::new(0.0, 0.0, 0.0),
            vec![2, 3],
            vec![Point::new(1.0, 0.0, 0.0),
                 Point::new(2.0, 0.0, 0.0),
                 Point::new(0.0, 1.0, 0.0),
                 Point::new(0.0, 2.0, 0.0),
                 Point::new(0.0, 3.0, 0.0)]);
        let mut iter = streamlines.into_iter();
        assert_eq!(iter.next().unwrap(),
                   [Point::new(1.0, 0.0, 0.0),
                    Point::new(2.0, 0.0, 0.0)]);
        assert_eq!(iter.next().unwrap(),
                   [Point::new(0.0, 1.0, 0.0),
                    Point::new(0.0, 2.0, 0.0),
                    Point::new(0.0, 3.0, 0.0)]);
        assert_eq!(iter.next(), None);
    }
}
