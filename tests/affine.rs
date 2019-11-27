#[cfg(feature = "nifti_images")]
extern crate nifti;
extern crate tempdir;
extern crate trk_io;

mod test;

#[cfg(feature = "nifti_images")]
mod nifti_tests {
    use nifti::{InMemNiftiObject, NiftiObject};
    use test::{get_random_trk_path, load_trk};
    use trk_io::{
        affine::{rasmm_to_trackvis, trackvis_to_rasmm},
        Affine, Affine4, CHeader, Header, Point, Translation, Writer,
    };

    #[test]
    fn test_complex_affine() {
        let header =
            InMemNiftiObject::from_file("data/complex_affine.nii.gz").unwrap().header().clone();
        let write_to = get_random_trk_path();

        {
            let mut writer = Writer::new(&write_to, Some(Header::from_nifti(&header))).unwrap();
            writer.apply_affine(&header.affine());
            writer.write(
                &[
                    Point::new(13.75, 27.90, 51.55),
                    Point::new(14.00, 27.95, 51.98),
                    Point::new(14.35, 28.05, 52.33),
                ][..],
            );
        }

        // Loading them back without the right transformation is not supposed to give back the same
        // points. Results are exactly the same as with DiPy.
        let streamlines = load_trk(&write_to).1.streamlines;
        let streamline = &streamlines[0];
        assert_eq!(streamline[0], Point::new(-82.54104, -25.178139, 37.788338));
        assert_eq!(streamline[1], Point::new(-81.933876, -25.032265, 38.850258));
        assert_eq!(streamline[2], Point::new(-81.07349, -24.765305, 39.70571));
    }

    #[test]
    fn test_qform_affine() {
        let header = InMemNiftiObject::from_file("data/qform.nii.gz").unwrap().header().clone();
        #[rustfmt::skip]
        assert_eq!(
            header.affine(),
            Affine4::new(
                -0.9375, 0.0, 0.0, 59.557503,
                0.0, 0.9375, 0.0, 73.172,
                0.0, 0.0, 3.0, 43.4291,
                0.0, 0.0, 0.0, 1.0,
            )
        );
    }

    #[test]
    fn test_simple_header_from_nifti() {
        let c_header = CHeader::from_nifti(
            [3, 100, 100, 100, 0, 0, 0, 0],
            [3.0, 1.1, 1.2, 1.3, 0.0, 0.0, 0.0, 0.0],
            [1.1, 0.0, 0.0, 10.0],
            [0.0, 1.2, 0.0, 11.0],
            [0.0, 0.0, 1.3, 12.0],
        );
        assert_eq!(c_header.dim, [100, 100, 100]);
        assert_eq!(c_header.voxel_size, [1.1, 1.2, 1.3]);
        #[rustfmt::skip]
        assert_eq!(
            c_header.vox_to_ras,
            [
                1.1, 0.0, 0.0, 10.0,
                0.0, 1.2, 0.0, 11.0,
                0.0, 0.0, 1.3, 12.0,
                0.0, 0.0, 0.0, 1.0,
            ]
        );
        assert_eq!(c_header.voxel_order, *b"RAS\0");
    }

    #[test]
    fn test_complex_header_from_nifti() {
        let nifti_header = InMemNiftiObject::from_file("data/3x3.nii.gz").unwrap().header().clone();
        let c_header = CHeader::from_nifti(
            nifti_header.dim,
            nifti_header.pixdim,
            nifti_header.srow_x,
            nifti_header.srow_y,
            nifti_header.srow_z,
        );
        assert_eq!(c_header.dim, [3, 3, 3]);
        assert_eq!(c_header.voxel_size, [2.0, 2.0, 2.0]);
        #[rustfmt::skip]
        assert_eq!(
            c_header.vox_to_ras,
            [
                -2.0, 0.0, 0.0, 90.0,
                0.0, 2.0, 0.0, -126.0,
                0.0, 0.0, 2.0, -72.0,
                0.0, 0.0, 0.0, 1.0,
            ]
        );
        assert_eq!(c_header.voxel_order, *b"LAS\0");

        let header = Header::from_nifti(&nifti_header);
        #[rustfmt::skip]
        assert_eq!(
            header.affine_to_rasmm,
            Affine::new(
                -1.0, 0.0, 0.0,
                0.0, 1.0, 0.0,
                0.0, 0.0, 1.0,
            )
        );
        assert_eq!(header.translation, Translation::new(91.0, -127.0, -73.0));
        assert_eq!(header.nb_streamlines, 0);
        assert_eq!(header.scalars_name.len(), 0);
        assert_eq!(header.properties_name.len(), 0);
    }

    #[test]
    fn test_complex_affine_from_nifti() {
        let nifti_header = InMemNiftiObject::from_file("data/3x3.nii.gz").unwrap().header().clone();
        #[rustfmt::skip]
        assert_eq!(
            trackvis_to_rasmm(&nifti_header),
            Affine4::new(
                -1.0, 0.0, 0.0, 91.0,
                0.0, 1.0, 0.0, -127.0,
                0.0, 0.0, 1.0, -73.0,
                0.0, 0.0, 0.0, 1.0,
            )
        );
        #[rustfmt::skip]
        assert_eq!(
            rasmm_to_trackvis(&nifti_header),
            Affine4::new(
                -1.0, 0.0, 0.0, 91.0,
                0.0, 1.0, 0.0, 127.0,
                0.0, 0.0, 1.0, 73.0,
                0.0, 0.0, 0.0, 1.0,
            )
        );
    }
}
