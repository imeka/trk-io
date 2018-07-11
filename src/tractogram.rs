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
