use crate::{Header, Points, Reader, Spacing};

/// Generator to read a TrackVis file, streamline per streamline.
///
/// Will never hold more than one streamline in memory.
pub struct VoxelSpaceReader {
    reader: Reader,
}

impl VoxelSpaceReader {
    /// Build a new `VoxelSpaceReader` object.
    ///
    /// * `path` - Path to TrackVis file
    /// * `spacing` - Spacing (pixel dimension `pixdim`) obtained from the `Header` or from a
    ///   reference image.
    pub fn new<P: AsRef<std::path::Path>>(path: P, spacing: Spacing) -> (Header, VoxelSpaceReader) {
        let reader = Reader::new(path).unwrap().to_voxel_space(spacing);
        let header = reader.header.clone();
        (header, VoxelSpaceReader { reader })
    }
}

impl Iterator for VoxelSpaceReader {
    type Item = Points;

    fn next(&mut self) -> Option<Points> {
        if let Some((points, _, _)) = self.reader.next() {
            return Some(points);
        } else {
            return None;
        }
    }
}
