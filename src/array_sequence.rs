
use std::mem;
use std::ops::Index;
use std::slice;
use std::vec::Vec;

#[derive(Clone, PartialEq)]
pub struct ArraySequence<T> {
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
        if self.it_idx < self.arr.offsets.len() - 1 {
            self.it_idx += 1;
            Some(&self.arr[self.it_idx - 1])
        } else {
            None
        }
    }
}

impl<'a, T> IntoIterator for &'a mut ArraySequence<T> {
    type Item = &'a mut [T];
    type IntoIter = ArraySequenceIteratorMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        let mut offsets = self.offsets.iter();
        let last_offset = *offsets.next().unwrap();
        ArraySequenceIteratorMut { data: &mut self.data, offsets, last_offset }
    }
}

pub struct ArraySequenceIteratorMut<'a, T: 'a> {
    data: &'a mut [T],
    offsets: slice::Iter<'a, usize>,
    last_offset: usize,
}

impl<'a, T> Iterator for ArraySequenceIteratorMut<'a, T> {
    type Item = &'a mut [T];

    fn next(&mut self) -> Option<Self::Item> {
        let current_offset = *self.offsets.next()?;
        let nb_elements = current_offset - self.last_offset;
        self.last_offset = current_offset;

        let data = mem::replace(&mut self.data, &mut []);
        let (slice, remaining_data) = data.split_at_mut(nb_elements);
        self.data = remaining_data;
        Some(slice)
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
        ArraySequence { offsets: vec![0], data: vec![] }
    }

    pub fn with_capacity(n: usize) -> ArraySequence<T> {
        ArraySequence { offsets: vec![0], data: Vec::with_capacity(n) }
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
        }).collect::<Vec<usize>>();

        // Check if `offsets` fits with the numbers of points in `data`
        let expected_points = *offsets.last().unwrap();
        if expected_points != data.len() {
            panic!(
                "`offsets` declares {} points but `data` contains {} points.",
                expected_points, data.len());
        }

        ArraySequence { offsets, data: data }
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
            self.offsets.push(self.data.len());
        }
    }

    pub fn len(&self) -> usize {
        self.offsets.len() - 1
    }

    /// Same as obj[i].len(), without building a slice
    pub fn length_of_array(&self, i: usize) -> usize {
        let current = unsafe { *self.offsets.get_unchecked(i) };
        let next = unsafe { *self.offsets.get_unchecked(i + 1) };
        next - current
    }

    pub fn filter<P>(&self, predicate: &mut P) -> ArraySequence<T>
        where P: FnMut(&[T]) -> bool,
              T: Clone
    {
        let mut new = ArraySequence::<T>::empty();
        for array in self {
            if predicate(array) {
                new.extend(array.iter().cloned());
            }
        }
        new
    }

    pub fn iter(&self) -> ArraySequenceIterator<T> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> ArraySequenceIteratorMut<T> {
        self.into_iter()
    }
}

impl<T> Extend<T> for ArraySequence<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.data.extend(iter);
        self.end_push();
    }
}

impl<T: Clone> ArraySequence<T> {
    pub fn extend_from_slice(&mut self, other: &[T]) {
        self.data.extend_from_slice(other);
        self.end_push();
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
    #[should_panic]
    fn test_new_not_enough() {
        ArraySequence::new(
            vec![2],
            vec![Point::new(1.0, 0.0, 0.0)]);
    }

    #[test]
    #[should_panic]
    fn test_new_too_much() {
        ArraySequence::new(
            vec![2],
            vec![Point::new(1.0, 0.0, 0.0),
                 Point::new(1.0, 0.0, 0.0),
                 Point::new(1.0, 0.0, 0.0)]);
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
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_iterator_mut() {
        let mut streamlines = ArraySequence::new(
            vec![2, 3, 2],
            vec![Point::new(1.0, 0.0, 0.0),
                 Point::new(2.0, 0.0, 0.0),
                 Point::new(0.0, 1.0, 0.0),
                 Point::new(0.0, 2.0, 0.0),
                 Point::new(0.0, 3.0, 0.0),
                 Point::new(0.0, 0.0, 1.0),
                 Point::new(0.0, 0.0, 2.0)]);
        for (i, streamline) in streamlines.iter_mut().enumerate() {
            for p in streamline {
                if i % 2 == 0 {
                    *p = Point::zeros();
                }
            }
        }
        let mut iter = streamlines.into_iter();
        assert_eq!(iter.next().unwrap(), [Point::zeros(), Point::zeros()]);
        assert_eq!(iter.next().unwrap(), [Point::new(0.0, 1.0, 0.0),
                                          Point::new(0.0, 2.0, 0.0),
                                          Point::new(0.0, 3.0, 0.0)]);
        assert_eq!(iter.next().unwrap(), [Point::zeros(), Point::zeros()]);
        assert_eq!(iter.next(), None);
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
        assert_eq!(arr.length_of_array(0), 10);
        assert_eq!(arr[0].len(), 10);
        assert_eq!(arr.offsets, vec![0, 10]);

        arr.extend(vec![11, 12, 13, 14, 15]);
        assert_eq!(arr.len(), 2);
        assert_eq!(arr.length_of_array(0), 10);
        assert_eq!(arr[0].len(), 10);
        assert_eq!(arr.length_of_array(1), 5);
        assert_eq!(arr[1].len(), 5);
        assert_eq!(arr.offsets, vec![0, 10, 15]);

        arr.extend_from_slice(&[20, 21, 22, 23]);
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[2].len(), 4);
        assert_eq!(arr.offsets, vec![0, 10, 15, 19]);
    }

    #[test]
    fn test_empty_push() {
        let mut arr = ArraySequence::<f64>::empty();
        assert_eq!(arr.len(), 0);
        assert_eq!(arr.offsets, vec![0]);

        // An `end_push` without any `push` should do nothing
        arr.end_push();

        assert_eq!(arr.len(), 0);
        assert_eq!(arr.offsets, vec![0]);
    }

    #[test]
    fn test_filter() {
        let p = Point::new(1.0, 1.0, 1.0);
        let arr = ArraySequence::new(
            vec![2, 3, 2, 3],
            vec![p * 1.0, p * 2.0,
                 p * 2.0, p * 3.0, p * 4.0,
                 p * 3.0, p * 4.0,
                 p * 4.0, p * 5.0, p * 6.0]);
        let filtered = arr.filter(&mut |arr: &[Point]| arr.len() == 3);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0], [p * 2.0, p * 3.0, p * 4.0]);
        assert_eq!(filtered[1], [p * 4.0, p * 5.0, p * 6.0]);

        // Ensure that arr is still usable
        assert_eq!(arr.len(), 4);
    }
}
