use std::ops::Range;

use nalgebra::Point3;

use ArraySequence;

pub type Point = Point3<f32>;
pub type Points = Vec<Point>;
pub type Streamlines = ArraySequence<Point>;

pub type Properties = Vec<f32>;
pub type Scalars = ArraySequence<f32>;

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

    pub fn item(&self, idx: usize) -> TractogramItem {
        let scalars = self.scalars.iter().map(|arr| arr[idx].to_vec()).collect();
        let properties = self.properties.iter().map(|v| v[idx]).collect();
        TractogramItem::new(
            self.streamlines[idx].to_vec(), scalars, properties
        )
    }
}

pub struct TractogramItem {
    pub streamline: Points,
    pub scalars: Vec<Vec<f32>>,
    pub properties: Vec<f32>
}

impl TractogramItem {
    pub fn new(
        streamline: Points,
        scalars: Vec<Vec<f32>>,
        properties: Vec<f32>
    ) -> TractogramItem {
        TractogramItem { streamline, scalars, properties }
    }

    pub fn from_slice(streamline: &[Point]) -> TractogramItem {
        let streamline = streamline.to_vec();
        TractogramItem { streamline, scalars: vec![], properties: vec![] }
    }
}

impl<'data> IntoIterator for &'data Tractogram {
    type Item = TractogramItem;
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
    type Item = TractogramItem;

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
