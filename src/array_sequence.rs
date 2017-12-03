
use std::ops::Index;
use std::vec::Vec;

#[derive(Clone, PartialEq)]
pub struct ArraySequence<T> {
    pub lengths: Vec<usize>,
    pub offsets: Vec<usize>,
    pub data: Vec<T>,
}

impl<'a, T> IntoIterator for &'a ArraySequence<T> {
    type Item = &'a [T];
    type IntoIter = ArraySequenceIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        ArraySequenceIterator {
            arr: self,
            it_idx: 0
        }
    }
}

pub struct ArraySequenceIterator<'a, T: 'a> {
    arr: &'a ArraySequence<T>,
    it_idx: usize
}

impl<'a, T> Iterator for ArraySequenceIterator<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        if self.it_idx < self.arr.lengths.len() {
            self.it_idx += 1;
            Some(&self.arr[self.it_idx - 1])
        }
        else {
            None
        }
    }
}

impl<T> Index<usize> for ArraySequence<T> {
    type Output = [T];

    fn index<'a>(&'a self, i: usize) -> &'a Self::Output {
        let start = unsafe { *self.offsets.get_unchecked(i) };
        let end = unsafe { *self.offsets.get_unchecked(i + 1) };
        &self.data[start..end]
    }
}

impl<T> ArraySequence<T> {
    pub fn new(
        lengths: Vec<usize>,
        data: Vec<T>
    ) -> ArraySequence<T> {
        // CumSum over lengths. [0, ..., ..., data.len()]
        // There's an additional offset at the end because we want a
        // branchless `Index<usize>` function.
        let offsets = [0].iter().chain(&lengths).scan(0, |state, x| {
            *state += *x;
            Some(*state)
        }).collect();

        ArraySequence { lengths, offsets, data: data }
    }

    pub fn len(&self) -> usize {
        self.lengths.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::RowVector3;
    pub type Point = RowVector3<f32>;

    #[test]
    fn test_integers() {
        let arr = ArraySequence::new(
            vec![2, 3, 2, 1],
            vec![4, 5, 6, 7, 8, 9, 10, 11]);
        assert_eq!(arr.len(), 4);
        assert_eq!(arr.offsets, vec![0, 2, 5, 7, 8]);
    }

    #[test]
    fn test_construction() {
        let arr = ArraySequence::new(
            vec![2, 3, 2],
            vec![Point::new(1.0, 0.0, 0.0),
                 Point::new(2.0, 0.0, 0.0),
                 Point::new(0.0, 1.0, 0.0),
                 Point::new(0.0, 2.0, 0.0),
                 Point::new(0.0, 3.0, 0.0),
                 Point::new(0.0, 0.0, 1.0),
                 Point::new(0.0, 0.0, 2.0)]);
        assert_eq!(arr.len(), 3);
        assert_eq!(arr.offsets, vec![0, 2, 5, 7]);
    }

    #[test]
    fn test_iterator() {
        let arr = ArraySequence::new(
            vec![2, 3],
            vec![Point::new(1.0, 0.0, 0.0),
                 Point::new(2.0, 0.0, 0.0),
                 Point::new(0.0, 1.0, 0.0),
                 Point::new(0.0, 2.0, 0.0),
                 Point::new(0.0, 3.0, 0.0)]);
        let mut iter = arr.into_iter();
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
