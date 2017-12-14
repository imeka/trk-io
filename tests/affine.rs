
#[cfg(feature = "use_nifti")] extern crate nifti;
extern crate trk_io;

#[cfg(feature = "use_nifti")]
mod nifti_tests {
    use nifti::{InMemNiftiObject, NiftiObject};
    use trk_io::{Affine, Affine4, CHeader, Header, Translation};
    use trk_io::affine::{rasmm_to_trackvis, trackvis_to_rasmm};

    #[test]
    fn test_simple_header_from_nifti() {
        let c_header = CHeader::from_nifti(
            [3, 100, 100, 100, 0, 0, 0, 0],
            [3.0, 1.1, 1.2, 1.3, 0.0, 0.0, 0.0, 0.0],
            [1.1, 0.0, 0.0, 10.0],
            [0.0, 1.2, 0.0, 11.0],
            [0.0, 0.0, 1.3, 12.0]
        );
        assert_eq!(c_header.dim, [100, 100, 100]);
        assert_eq!(c_header.voxel_size, [1.1, 1.2, 1.3]);
        assert_eq!(c_header.vox_to_ras, [1.1, 0.0, 0.0, 10.0,
                                         0.0, 1.2, 0.0, 11.0,
                                         0.0, 0.0, 1.3, 12.0,
                                         0.0, 0.0, 0.0, 1.0]);
        assert_eq!(c_header.voxel_order, *b"RAS\0");
    }

    #[test]
    fn test_complex_header_from_nifti() {
        let nifti_header = InMemNiftiObject::from_file("data/3x3.nii.gz")
            .unwrap().header().clone();

        let c_header = CHeader::from_nifti(
            nifti_header.dim,
            nifti_header.pixdim,
            nifti_header.srow_x,
            nifti_header.srow_y,
            nifti_header.srow_z);
        assert_eq!(c_header.dim, [3, 3, 3]);
        assert_eq!(c_header.voxel_size, [2.0, 2.0, 2.0]);
        assert_eq!(c_header.vox_to_ras, [-2.0, 0.0, 0.0, 90.0,
                                          0.0, 2.0, 0.0, -126.0,
                                          0.0, 0.0, 2.0, -72.0,
                                          0.0, 0.0, 0.0, 1.0]);
        assert_eq!(c_header.voxel_order, *b"LAS\0");

        let header = Header::from_nifti(nifti_header);
        assert_eq!(header.affine, Affine::new(-1.0, 0.0, 0.0,
                                               0.0, 1.0, 0.0,
                                               0.0, 0.0, 1.0));
        assert_eq!(header.translation, Translation::new(91.0, -127.0, -73.0));
        assert_eq!(header.nb_streamlines, 0);
        assert_eq!(header.scalars.len(), 0);
        assert_eq!(header.properties.len(), 0);
    }

    #[test]
    fn test_complex_affine_from_nifti() {
        let nifti_header = InMemNiftiObject::from_file("data/3x3.nii.gz")
            .unwrap().header().clone();
        assert_eq!(trackvis_to_rasmm(&nifti_header),
                   Affine4::new(-1.0, 0.0, 0.0, 91.0,
                                0.0, 1.0, 0.0, -127.0,
                                0.0, 0.0, 1.0, -73.0,
                                0.0, 0.0, 0.0, 1.0));
        assert_eq!(rasmm_to_trackvis(&nifti_header),
                   Affine4::new(-1.0, 0.0, 0.0, 91.0,
                                0.0, 1.0, 0.0, 127.0,
                                0.0, 0.0, 1.0, 73.0,
                                0.0, 0.0, 0.0, 1.0));
    }
}
