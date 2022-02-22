/* The MIT License

Copyright (c) 2009-2014 Matthew Brett <matthew.brett@gmail.com>
Copyright (c) 2010-2013 Stephan Gerhard <git@unidesign.ch>
Copyright (c) 2006-2014 Michael Hanke <michael.hanke@gmail.com>
Copyright (c) 2011 Christian Haselgrove <christian.haselgrove@umassmed.edu>
Copyright (c) 2010-2011 Jarrod Millman <jarrod.millman@gmail.com>
Copyright (c) 2011-2014 Yaroslav Halchenko <debian@onerussian.com>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software. */

/* The ideas in this file were taken from the NiBabel project, which is MIT
licensed. The port to Rust has been done by:

Copyright (c) 2017-2022 Nil Goyette <nil.goyette@gmail.com>
*/

use nalgebra::{RowVector3, Vector4};
#[cfg(feature = "nifti_images")]
use ndarray::{ArrayBase, Axis, DataMut, Dimension, Zip};
#[cfg(feature = "nifti_images")]
use nifti::DataElement;

use crate::{Affine, Affine4, Translation};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    Normal,
    Reversed,
}

impl Direction {
    fn to_f32(&self) -> f32 {
        if *self == Direction::Normal {
            1.0
        } else {
            -1.0
        }
    }
}

pub type Orientation = (usize, Direction);
pub type Orientations = [Orientation; 3];

/// Axis direction codes for affine `affine`
pub fn affine_to_axcodes(affine: &Affine) -> String {
    let orientations = io_orientations(affine);
    orientations_to_axcodes(orientations)
}

/// Orientation of input axes in terms of output axes for `affine`
///
/// Valid for an affine transformation from `p` dimensions to `q` dimensions
/// (`affine.shape == (q + 1, p + 1)`).
///
/// The calculated orientations can be used to transform associated arrays to
/// best match the output orientations. If `p` > `q`, then some of the output
/// axes should be considered dropped in this orientation.
pub fn io_orientations(affine: &Affine) -> Orientations {
    // Extract the underlying rotation, zoom, shear matrix
    let rzs2 = affine.component_mul(affine);
    let mut zooms = RowVector3::new(
        (rzs2[0] + rzs2[1] + rzs2[2]).sqrt(),
        (rzs2[3] + rzs2[4] + rzs2[5]).sqrt(),
        (rzs2[6] + rzs2[7] + rzs2[8]).sqrt(),
    );

    // Zooms can be zero, in which case all elements in the column are zero,
    // and we can leave them as they are
    zooms.apply(|z| if z == 0.0 { 1.0 } else { z });

    #[rustfmt::skip]
    let rs = Affine::new(
        affine[0] / zooms[0], affine[3] / zooms[1], affine[6] / zooms[2],
        affine[1] / zooms[0], affine[4] / zooms[1], affine[7] / zooms[2],
        affine[2] / zooms[0], affine[5] / zooms[1], affine[8] / zooms[2],
    );

    // Transform below is polar decomposition, returning the closest shearless
    // matrix R to RS
    let svd = rs.svd(true, true);
    let (u, s, v_t) = (svd.u.unwrap(), svd.singular_values, svd.v_t.unwrap());

    // Threshold the singular values to determine the rank.
    let tol = s.as_slice().iter().cloned().fold(0.0, f32::max) * 3.0 * f32::EPSILON;

    let s = Affine::from_rows(&[s.transpose(), s.transpose(), s.transpose()]);
    let u = u.zip_map(&s, |u, s| if s > tol { u } else { 0.0 });
    let v_t = v_t.zip_map(&s, |v, s| if s > tol { v } else { 0.0 });

    // The matrix R is such that np.dot(R, R.T) is projection onto the columns
    // of P[.., keep] and np.dot(R.T, R) is projection onto the rows of
    // Qs[keep]. R (== np.dot(R, np.eye(p))) gives rotation of the unit input
    // vectors to output coordinates. Therefore, the row index of abs max
    // R[.., N], is the output axis changing most as input axis N changes. In
    // case there are ties, we choose the axes iteratively, removing used axes
    // from consideration as we go
    let mut r = u * v_t;

    let mut orientations = [(0, Direction::Normal), (0, Direction::Normal), (0, Direction::Normal)];
    for c in 0..3 {
        let mut argmax = 0;
        let mut max = 0.0;
        let mut sign_max = 0.0;
        for (i, e) in r.column(c).iter().enumerate() {
            let e_abs = e.abs();
            if e_abs > max {
                argmax = i;
                max = e_abs;
                sign_max = *e;
            }
        }

        if sign_max >= 0.0 {
            orientations[c] = (argmax, Direction::Normal);
        } else {
            orientations[c] = (argmax, Direction::Reversed);
        }

        // Remove the identified axis from further consideration, by zeroing
        // out the corresponding row in R
        for e in r.column_mut(c).iter_mut() {
            *e = 0.0;
        }
    }

    orientations
}

/// Convert orientation `orientations` to labels for axis directions
pub fn orientations_to_axcodes(orientations: Orientations) -> String {
    let labels = [
        ("L".to_string(), "R".to_string()),
        ("P".to_string(), "A".to_string()),
        ("I".to_string(), "S".to_string()),
    ];

    orientations
        .iter()
        .map(|&(ref axis, ref direction)| {
            if *direction == Direction::Normal {
                labels[*axis].1.clone()
            } else {
                labels[*axis].0.clone()
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

/// Convert axis codes `axcodes` to an orientation
pub fn axcodes_to_orientations(axcodes: &str) -> Orientations {
    let labels = [('L', 'R'), ('P', 'A'), ('I', 'S')];
    let mut orientations = [(0, Direction::Normal), (0, Direction::Normal), (0, Direction::Normal)];
    for (code_idx, code) in axcodes.chars().enumerate() {
        for (label_idx, codes) in labels.iter().enumerate() {
            if code == codes.0 {
                orientations[code_idx] = (label_idx, Direction::Reversed);
            } else if code == codes.1 {
                orientations[code_idx] = (label_idx, Direction::Normal);
            }
        }
    }
    orientations
}

/// Return the orientation that transforms from `start_orientations` to
/// `end_orientations`
pub fn orientations_transform(
    start_orientations: &Orientations,
    end_orientations: &Orientations,
) -> Orientations {
    let mut result = [(0, Direction::Normal), (0, Direction::Normal), (0, Direction::Normal)];
    for (end_in_idx, &(ref end_out_idx, ref end_flip)) in end_orientations.iter().enumerate() {
        for (start_in_idx, &(ref start_out_idx, ref start_flip)) in
            start_orientations.iter().enumerate()
        {
            if end_out_idx == start_out_idx {
                if start_flip == end_flip {
                    result[start_in_idx] = (end_in_idx, Direction::Normal)
                } else {
                    result[start_in_idx] = (end_in_idx, Direction::Reversed)
                }
                break;
            }
        }
    }
    result
}

#[cfg(feature = "nifti_images")]
/// Apply transformations implied by `orientations` to the first n axes of the array `arr`.
pub fn apply_orientation<S, A, D>(
    mut arr: ArrayBase<S, D>,
    orientations: Orientations,
) -> ArrayBase<S, D>
where
    S: DataMut<Elem = A>,
    A: DataElement,
    D: Dimension,
{
    // Apply orientation transformations
    for (axis, &(_, direction)) in orientations.iter().enumerate() {
        if direction == Direction::Reversed {
            Zip::from(arr.lanes_mut(Axis(axis))).for_each(|mut arr| {
                let n = arr.len();
                for i in 0..n / 2 {
                    let tmp = arr[n - 1 - i];
                    arr[n - 1 - i] = arr[i];
                    arr[i] = tmp;
                }
            });
        }
    }

    // Orientation indicates the transpose that has occurred - we reverse it
    let mut x = (orientations[0].0, 0);
    let mut y = (orientations[1].0, 1);
    let mut z = (orientations[2].0, 2);
    if x > y {
        std::mem::swap(&mut x, &mut y);
    }
    if y > z {
        std::mem::swap(&mut y, &mut z);
    }
    if x > y {
        std::mem::swap(&mut x, &mut y);
    }

    let mut axes = arr.raw_dim();
    axes[0] = x.1;
    axes[1] = y.1;
    axes[2] = z.1;
    arr.permuted_axes(axes)
}

/// Affine transform reversing transforms implied in `orientations`
///
/// Imagine you have an array `arr` of shape `shape`, and you apply the
/// transforms implied by `orientations`, to get `tarr`. `tarr` may have a
/// different shape `shape_prime`. This routine returns the affine that will
/// take an array coordinate for `tarr` and give you the corresponding array
/// coordinate in `arr`.
pub fn inverse_orientations_affine(orientations: &Orientations, dim: [i16; 3]) -> Affine4 {
    let mut undo_reorder = Affine4::zeros();
    for (i, &(j, _)) in orientations.iter().enumerate() {
        undo_reorder[(i, j)] = 1.0;
    }
    undo_reorder[(3, 3)] = 1.0;

    let center = Translation::new(
        -(dim[0] - 1) as f32 / 2.0,
        -(dim[1] - 1) as f32 / 2.0,
        -(dim[2] - 1) as f32 / 2.0,
    );
    let mut undo_flip = Affine4::from_diagonal(&Vector4::new(
        orientations[0].1.to_f32(),
        orientations[1].1.to_f32(),
        orientations[2].1.to_f32(),
        1.0,
    ));
    undo_flip[(0, 3)] = undo_flip[(0, 0)] * center[0] - center[0];
    undo_flip[(1, 3)] = undo_flip[(1, 1)] * center[1] - center[1];
    undo_flip[(2, 3)] = undo_flip[(2, 2)] * center[2] - center[2];

    undo_flip * undo_reorder
}

#[cfg(test)]
mod tests {
    use super::*;

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
        use ndarray::{arr3, Array1, Array4};

        let arr = (0..24).collect::<Array1<_>>().into_shape((2, 3, 4)).unwrap();
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

        let psr_to_las =
            [(1, Direction::Reversed), (2, Direction::Normal), (0, Direction::Reversed)];
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
        let psr_to_ilp =
            [(2, Direction::Normal), (0, Direction::Reversed), (1, Direction::Reversed)];
        let gt = arr3(&[
            [[11, 23], [10, 22], [9, 21], [8, 20]],
            [[7, 19], [6, 18], [5, 17], [4, 16]],
            [[3, 15], [2, 14], [1, 13], [0, 12]],
        ]);
        assert_eq!(apply_orientation(arr.clone(), psr_to_ilp), gt);

        // 4D test. The 4th axis should never be reoriented.
        let arr = arr.mapv(|v| v as f32);
        let gt = gt.mapv(|v| v as f32);
        let mut arr4 = Array4::<f32>::zeros((2, 3, 4, 3));
        for mut volume in arr4.axis_iter_mut(Axis(3)) {
            volume.assign(&arr);
        }
        let answer = apply_orientation(arr4, psr_to_ilp);
        for volume in answer.axis_iter(Axis(3)) {
            assert_eq!(volume, gt);
        }
    }
}
