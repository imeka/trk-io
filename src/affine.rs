
#[cfg(feature = "use_nifti")] 
use nifti::NiftiHeader;
use nalgebra::U3;

#[cfg(feature = "use_nifti")]  use CHeader;
use {Affine, Affine4, Translation};

pub fn get_affine_and_translation(
    affine: &Affine4
) -> (Affine, Translation) {
    let translation = Translation::new(
        affine[12], affine[13], affine[14]);
    let affine = affine.fixed_slice::<U3, U3>(0, 0).into_owned();
    (affine, translation)
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
