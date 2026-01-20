#[cfg(feature = "nifti_images")]
use ndarray::Axis;

#[cfg(feature = "nifti_images")]
use trk_io::orientation::apply_orientation;
use trk_io::{
    Affine,
    orientation::{
        Direction, affine_to_axcodes, axcodes_to_orientations, orientations_to_axcodes,
        orientations_transform,
    },
};

#[test]
fn test_affine_to_axcodes() {
    assert_eq!(affine_to_axcodes(&Affine::identity()), "RAS".to_string());

    let affine = Affine::new(0.0, 1.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    assert_eq!(affine_to_axcodes(&affine), "PRS".to_string());
    assert_eq!(affine_to_axcodes(&affine), "PRS".to_string());
}

#[test]
fn test_axcodes_to_orientations() {
    assert_eq!(
        axcodes_to_orientations("RAS"),
        [(0, Direction::Normal), (1, Direction::Normal), (2, Direction::Normal)]
    );
    assert_eq!(
        axcodes_to_orientations("LPI"),
        [(0, Direction::Reversed), (1, Direction::Reversed), (2, Direction::Reversed)]
    );
    assert_eq!(
        axcodes_to_orientations("SAR"),
        [(2, Direction::Normal), (1, Direction::Normal), (0, Direction::Normal)]
    );
    assert_eq!(
        axcodes_to_orientations("AIR"),
        [(1, Direction::Normal), (2, Direction::Reversed), (0, Direction::Normal)]
    );
}

#[test]
fn test_orientations_to_axcodes() {
    assert_eq!(
        orientations_to_axcodes([
            (0, Direction::Normal),
            (1, Direction::Normal),
            (2, Direction::Normal)
        ]),
        "RAS"
    );
    assert_eq!(
        orientations_to_axcodes([
            (0, Direction::Reversed),
            (1, Direction::Reversed),
            (2, Direction::Reversed)
        ]),
        "LPI"
    );
    assert_eq!(
        orientations_to_axcodes([
            (2, Direction::Reversed),
            (1, Direction::Reversed),
            (0, Direction::Reversed)
        ]),
        "IPL"
    );
    assert_eq!(
        orientations_to_axcodes([
            (1, Direction::Normal),
            (2, Direction::Reversed),
            (0, Direction::Normal)
        ]),
        "AIR"
    );
}

#[test]
fn test_orientations_transform() {
    assert_eq!(
        orientations_transform(
            &[(0, Direction::Normal), (1, Direction::Normal), (2, Direction::Reversed)],
            &[(1, Direction::Normal), (0, Direction::Normal), (2, Direction::Normal)]
        ),
        [(1, Direction::Normal), (0, Direction::Normal), (2, Direction::Reversed)]
    );
    assert_eq!(
        orientations_transform(
            &[(0, Direction::Normal), (1, Direction::Normal), (2, Direction::Normal)],
            &[(2, Direction::Normal), (0, Direction::Reversed), (1, Direction::Normal)]
        ),
        [(1, Direction::Reversed), (2, Direction::Normal), (0, Direction::Normal)]
    );
}

#[cfg(feature = "nifti_images")]
#[test]
fn test_apply_orientation() {
    use ndarray::{Array1, Array4, arr3};

    let arr = (0..24).collect::<Array1<_>>().into_shape_with_order((2, 3, 4)).unwrap();
    let lsp_to_las = [(0, Direction::Normal), (2, Direction::Normal), (1, Direction::Reversed)];
    assert_eq!(
        apply_orientation(arr.clone(), lsp_to_las),
        arr3(&[
            [[3, 7, 11], [2, 6, 10], [1, 5, 9], [0, 4, 8]],
            [[15, 19, 23], [14, 18, 22], [13, 17, 21], [12, 16, 20]],
        ])
    );
    let lsp_to_psr = [(2, Direction::Reversed), (1, Direction::Normal), (0, Direction::Normal)];
    assert_eq!(
        apply_orientation(arr.clone(), lsp_to_psr),
        arr3(&[
            [[12, 0], [16, 4], [20, 8]],
            [[13, 1], [17, 5], [21, 9]],
            [[14, 2], [18, 6], [22, 10]],
            [[15, 3], [19, 7], [23, 11]],
        ])
    );
    let lsp_to_ilp = [(1, Direction::Normal), (0, Direction::Reversed), (2, Direction::Normal)];
    assert_eq!(
        apply_orientation(arr.clone(), lsp_to_ilp),
        arr3(&[
            [[8, 9, 10, 11], [20, 21, 22, 23]],
            [[4, 5, 6, 7], [16, 17, 18, 19]],
            [[0, 1, 2, 3], [12, 13, 14, 15]],
        ])
    );

    let psr_to_las = [(1, Direction::Reversed), (2, Direction::Normal), (0, Direction::Reversed)];
    assert_eq!(
        apply_orientation(arr.clone(), psr_to_las),
        arr3(&[
            [[15, 19, 23], [3, 7, 11]],
            [[14, 18, 22], [2, 6, 10]],
            [[13, 17, 21], [1, 5, 9]],
            [[12, 16, 20], [0, 4, 8]],
        ])
    );
    let psr_to_psr = [(0, Direction::Normal), (1, Direction::Normal), (2, Direction::Normal)];
    assert_eq!(apply_orientation(arr.clone(), psr_to_psr), arr);
    let psr_to_ilp = [(2, Direction::Normal), (0, Direction::Reversed), (1, Direction::Reversed)];
    let gt = arr3(&[
        [[11, 23], [10, 22], [9, 21], [8, 20]],
        [[7, 19], [6, 18], [5, 17], [4, 16]],
        [[3, 15], [2, 14], [1, 13], [0, 12]],
    ]);
    assert_eq!(apply_orientation(arr.clone(), psr_to_ilp), gt);

    // 4D test. The 4th axis should never be reoriented.
    let arr = arr.mapv(|v| v as f32);
    let gt = gt.mapv(|v| v as f32);
    let mut arr4 = Array4::<f32>::zeros((2, 3, 4, 10));
    for mut volume in arr4.axis_iter_mut(Axis(3)) {
        volume.assign(&arr);
    }
    let answer = apply_orientation(arr4, psr_to_ilp);
    for volume in answer.axis_iter(Axis(3)) {
        assert_eq!(volume, gt);
    }
}
