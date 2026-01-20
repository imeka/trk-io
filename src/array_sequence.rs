use std::{
    mem,
    ops::{Index, Range},
    slice,
    vec::Vec,
};

#[derive(Clone, PartialEq)]
pub struct ArraySequence<T> {
    pub offsets: Vec<usize>,
    pub data: Vec<T>,
}

impl<'a, T> IntoIterator for &'a ArraySequence<T> {
    type Item = &'a [T];
    type IntoIter = ArraySequenceIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        ArraySequenceIterator { arr: self, index: 0..self.len() }
    }
}

pub struct ArraySequenceIterator<'a, T: 'a> {
    arr: &'a ArraySequence<T>,
    index: Range<usize>,
}

impl<'a, T> Iterator for ArraySequenceIterator<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.index.next()?;
        Some(&self.arr[idx])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.arr.len()))
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

impl<'data, T> ExactSizeIterator for ArraySequenceIterator<'data, T> {}

impl<'data, T> DoubleEndedIterator for ArraySequenceIterator<'data, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let idx = self.index.next_back()?;
        Some(&self.arr[idx])
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

impl<T> Default for ArraySequence<T> {
    fn default() -> Self {
        ArraySequence::empty()
    }
}

impl<T> ArraySequence<T> {
    pub fn empty() -> ArraySequence<T> {
        ArraySequence { offsets: vec![0], data: vec![] }
    }

    pub fn with_capacity(n: usize) -> ArraySequence<T> {
        ArraySequence { offsets: vec![0], data: Vec::with_capacity(n) }
    }

    pub fn new(lengths: Vec<usize>, data: Vec<T>) -> ArraySequence<T> {
        // CumSum over lengths. [0, ..., ..., data.len()]
        // There's an additional offset at the end because we want a
        // branchless `Index<usize>` function.
        let offsets = [0]
            .iter()
            .chain(&lengths)
            .scan(0, |state, x| {
                *state += *x;
                Some(*state)
            })
            .collect::<Vec<usize>>();

        // Check if `offsets` fits with the numbers of points in `data`
        let expected_points = *offsets.last().unwrap();
        if expected_points != data.len() {
            panic!(
                "`offsets` declares {} points but `data` contains {} points.",
                expected_points,
                data.len()
            );
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

    /// Returns `true` if the array contains no elements.
    ///
    /// The array will be considered non empty if there was one or more
    /// `push()`, even without an `end_push()`. Use `len()` instead to ignore
    /// all pushed elements.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
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
    where
        P: FnMut(&[T]) -> bool,
        T: Clone,
    {
        let mut new = ArraySequence::<T>::empty();
        for array in self {
            if predicate(array) {
                new.extend(array.iter().cloned());
            }
        }
        new
    }

    pub fn iter(&self) -> ArraySequenceIterator<'_, T> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> ArraySequenceIteratorMut<'_, T> {
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
