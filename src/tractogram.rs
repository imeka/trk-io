use std::ops::Range;

use nalgebra::Point3;

use ArraySequence;

pub type Point = Point3<f32>;
pub type Points = Vec<Point>;
pub type Streamlines = ArraySequence<Point>;

pub type Properties = Vec<f32>;
pub type Scalars = ArraySequence<f32>;

pub type TractogramItem = (Points, Vec<Vec<f32>>, Vec<f32>);
pub type RefTractogramItem<'data> = (
    &'data[Point],
    Vec<&'data[f32]>,
    Vec<f32>
);

#[derive(Clone, PartialEq)]
pub struct Tractogram {
    pub streamlines: Streamlines,
    pub scalars: Vec<Scalars>,
    pub properties: Vec<Properties>
}

impl Tractogram {
    pub fn new(
        streamlines: Streamlines,
        scalars: Vec<Scalars>,
        properties: Vec<Properties>
    ) -> Tractogram {
        Tractogram { streamlines, scalars, properties }
    }

    pub fn item(&self, idx: usize) -> RefTractogramItem {
        let scalars = self.scalars.iter().map(|arr| &arr[idx]).collect();
        let properties = self.properties.iter().map(|v| v[idx]).collect();
        (&self.streamlines[idx], scalars, properties)
    }
}

impl<'data> IntoIterator for &'data Tractogram {
    type Item = RefTractogramItem<'data>;
    type IntoIter = TractogramIterator<'data>;

    fn into_iter(self) -> Self::IntoIter {
        TractogramIterator {
            tractogram: self,
            index: 0..self.streamlines.len()
        }
    }
}

pub struct TractogramIterator<'data> {
    tractogram: &'data Tractogram,
    index: Range<usize>
}

impl<'data> Iterator for TractogramIterator<'data> {
    type Item = RefTractogramItem<'data>;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.index.next()?;
        Some(self.tractogram.item(idx))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.tractogram.streamlines.len()))
    }
}

impl<'data> ExactSizeIterator for TractogramIterator<'data> {}

impl<'data> DoubleEndedIterator for TractogramIterator<'data> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let idx = self.index.next_back()?;
        Some(self.tractogram.item(idx))
    }
}
