use nalgebra::{Matrix3, Matrix4, Scalar, Vector3};
#[cfg(feature = "nifti_images")]
use nifti::NiftiHeader;

#[cfg(feature = "nifti_images")]
use crate::{Affine4, CHeader};

pub fn get_affine_and_translation<T>(affine: &Matrix4<T>) -> (Matrix3<T>, Vector3<T>)
where
    T: Copy + Scalar,
{
    let translation = Vector3::<T>::new(affine[12], affine[13], affine[14]);
    let affine = affine.fixed_view::<3, 3>(0, 0).into_owned();
    (affine, translation)
}

#[cfg(feature = "nifti_images")]
pub fn rasmm_to_trackvis(h: &NiftiHeader) -> Affine4 {
    trackvis_to_rasmm(h).try_inverse().unwrap()
}

#[cfg(feature = "nifti_images")]
pub fn trackvis_to_rasmm(h: &NiftiHeader) -> Affine4 {
    let c_header = CHeader::from_nifti(h.dim, h.pixdim, h.srow_x, h.srow_y, h.srow_z);
    c_header.get_affine_to_rasmm()
}
