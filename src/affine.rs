
#[cfg(feature = "use_nifti")]
use nifti::NiftiHeader;
use nalgebra::{Matrix3, Matrix4, Vector3, Scalar, U3};

#[cfg(feature = "use_nifti")] use {Affine4, CHeader};

pub fn get_affine_and_translation<T: Scalar>(
    affine: &Matrix4<T>
) -> (Matrix3<T>, Vector3<T>) {
    let translation = Vector3::<T>::new(
        affine[12], affine[13], affine[14]);
    let affine = affine.fixed_slice::<U3, U3>(0, 0).into_owned();
    (affine, translation)
}

#[cfg(feature = "use_nifti")]
pub fn raw_affine_from_nifti(h: &NiftiHeader) -> Affine4 {
    Affine4::new(h.srow_x[0], h.srow_x[1], h.srow_x[2], h.srow_x[3],
                 h.srow_y[0], h.srow_y[1], h.srow_y[2], h.srow_y[3],
                 h.srow_z[0], h.srow_z[1], h.srow_z[2], h.srow_z[3],
                 0.0, 0.0, 0.0, 1.0)
}

#[cfg(feature = "use_nifti")]
pub fn rasmm_to_trackvis(h: &NiftiHeader) -> Affine4 {
    trackvis_to_rasmm(h).try_inverse().unwrap()
}

#[cfg(feature = "use_nifti")]
pub fn trackvis_to_rasmm(h: &NiftiHeader) -> Affine4 {
    let c_header = CHeader::from_nifti(
        h.dim, h.pixdim, h.srow_x, h.srow_y, h.srow_z);
    c_header.get_affine()
}
