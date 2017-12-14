
use nifti::NiftiHeader;

use {Affine4};
use cheader::{CHeader};

pub fn rasmm_to_trackvis(h: &NiftiHeader) -> Affine4 {
    trackvis_to_rasmm(h).try_inverse().unwrap()
}

pub fn trackvis_to_rasmm(h: &NiftiHeader) -> Affine4 {
    let c_header = CHeader::from_nifti(
        h.dim, h.pixdim, h.srow_x, h.srow_y, h.srow_z);
    c_header.get_affine()
}
