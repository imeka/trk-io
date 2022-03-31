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
    pub fn to_f32(&self) -> f32 {
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

/// Convert `orientations` to labels for axis directions
///
/// The "identity" orientation is RAS. Thus, calling this function with
///
/// `[(0, Direction::Normal), (1, Direction::Normal), (2, Direction::Normal)]`
///
/// would return "RAS".
pub fn orientations_to_axcodes(orientations: Orientations) -> String {
    let labels = [['R', 'L'], ['A', 'P'], ['S', 'I']];
    orientations
        .into_iter()
        .map(|(axis, direction)| labels[axis][(direction == Direction::Reversed) as usize])
        .collect()
}

/// Convert axis codes `axcodes` to an orientation
///
/// The "identity" orientation is RAS. Thus, calling this function with "RAS" would return
///
/// `[(0, Direction::Normal), (1, Direction::Normal), (2, Direction::Normal)]`
///
/// If the caller has a different default orientation, like LAS, he must reverse the `Direction` of
/// the `0` axis.
pub fn axcodes_to_orientations(axcodes: &str) -> Orientations {
    let labels = [('R', 'L'), ('A', 'P'), ('S', 'I')];
    let mut orientations = [(0, Direction::Normal), (0, Direction::Normal), (0, Direction::Normal)];
    for (code_idx, code) in axcodes.chars().enumerate() {
        for (label_idx, codes) in labels.iter().enumerate() {
            if code == codes.0 {
                orientations[code_idx] = (label_idx, Direction::Normal);
            } else if code == codes.1 {
                orientations[code_idx] = (label_idx, Direction::Reversed);
            }
        }
    }
    orientations
}

/// Return the orientation that transforms from `start_orientations` to `end_orientations`.
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
    // The following block is simply an argsort of 3 numbers.
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
    for i in 3..arr.ndim() {
        axes[i] = i;
    }
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
