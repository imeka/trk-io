
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
    pub fn empty() -> ArraySequence<T> {
        ArraySequence { lengths: vec![], offsets: vec![0], data: vec![] }
    }

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

    pub fn push(&mut self, val: T) {
        self.data.push(val);
    }

    pub fn nb_push_done(&self) -> usize {
        self.data.len() - self.offsets.last().unwrap()
    }

    pub fn end_push(&mut self) {
        let nb = self.nb_push_done();
        if nb > 0 {
            self.lengths.push(nb);
            self.offsets.push(self.data.len());
        }
    }

    pub fn extend<I>(&mut self, iter: I)
        where I: IntoIterator<Item = T>
    {
        self.data.extend(iter);
        self.end_push();
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

    #[test]
    fn test_dynamic() {
        let mut arr = ArraySequence::empty();
        for i in 0..10 {
            assert_eq!(arr.nb_push_done(), i);
            arr.push(i);
            assert_eq!(arr.nb_push_done(), i + 1);
        }
        arr.end_push();
        assert_eq!(arr.nb_push_done(), 0);

        assert_eq!(arr.len(), 1);
        assert_eq!(arr.lengths, vec![10]);
        assert_eq!(arr.offsets, vec![0, 10]);

        arr.extend(vec![11, 12, 13, 14, 15]);
        assert_eq!(arr.len(), 2);
        assert_eq!(arr.lengths, vec![10, 5]);
        assert_eq!(arr.offsets, vec![0, 10, 15]);
    }

    #[test]
    fn test_empty_push() {
        let mut arr = ArraySequence::<f64>::empty();
        assert_eq!(arr.len(), 0);
        assert_eq!(arr.lengths, vec![]);
        assert_eq!(arr.offsets, vec![0]);

        // An `end_push` without any `push` should do nothing
        arr.end_push();

        assert_eq!(arr.len(), 0);
        assert_eq!(arr.lengths, vec![]);
        assert_eq!(arr.offsets, vec![0]);
    }
}
