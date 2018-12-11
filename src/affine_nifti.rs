use nalgebra::{Matrix3, Matrix4, Quaternion, RowVector4, SymmetricEigen, Vector3, U1};
use nifti::NiftiHeader;

use affine::get_affine_and_translation;
use Affine4;

const QUARTERNION_THRESHOLD: f64 = ::std::f64::EPSILON * 3.0;

pub fn get_nifti_affine(header: &NiftiHeader) -> Affine4 {
    if header.sform_code != 0 {
        get_sform(header)
    } else if header.qform_code != 0 {
        get_qform(header)
    } else {
        get_base_affine(header)
    }
}

fn get_sform(header: &NiftiHeader) -> Affine4 {
    Affine4::new(
        header.srow_x[0], header.srow_x[1], header.srow_x[2], header.srow_x[3],
        header.srow_y[0], header.srow_y[1], header.srow_y[2], header.srow_y[3],
        header.srow_z[0], header.srow_z[1], header.srow_z[2], header.srow_z[3],
        0.0, 0.0, 0.0, 1.0,
    )
}

/// Return 4x4 affine matrix from qform parameters in header
fn get_qform(header: &NiftiHeader) -> Affine4 {
    if header.pixdim[1] < 0.0 || header.pixdim[2] < 0.0 || header.pixdim[3] < 0.0 {
        panic!("All spacings (pixdim) should be positive");
    }
    if header.pixdim[0].abs() != 1.0 {
        panic!("qfac (pixdim[0]) should be 1 or -1");
    }

    let quaternion = get_qform_quaternion(header);
    let r = quaternion_to_affine(quaternion);
    let s = Matrix3::from_diagonal(&Vector3::new(
        header.pixdim[1] as f64,
        header.pixdim[2] as f64,
        (header.pixdim[3] * header.pixdim[0]) as f64,
    ));
    let m = r * s;
    Affine4::new(
        m[0] as f32, m[3] as f32, m[6] as f32, header.quatern_x,
        m[1] as f32, m[4] as f32, m[7] as f32, header.quatern_y,
        m[2] as f32, m[5] as f32, m[8] as f32, header.quatern_z,
        0.0, 0.0, 0.0, 1.0,
    )
}

/// Get affine from basic (shared) header fields
///
/// Note that we get the translations from the center of the image.
pub fn get_base_affine(header: &NiftiHeader) -> Affine4 {
    let d = header.dim[0] as usize;
    shape_zoom_affine(&header.dim[1..d + 1], &header.pixdim[1..d + 1])
}

/// Get affine implied by given shape and zooms
///
/// We get the translations from the center of the image (implied by `shape`).
fn shape_zoom_affine(shape: &[u16], spacing: &[f32]) -> Affine4 {
    // Get translations from center of image
    let origin = Vector3::new(
        (shape[0] as f32 - 1.0) / 2.0,
        (shape[1] as f32 - 1.0) / 2.0,
        (shape[2] as f32 - 1.0) / 2.0,
    );
    let spacing = [-spacing[0] as f32, spacing[1] as f32, spacing[2] as f32];
    Affine4::new(
        spacing[0], 0.0, 0.0, -origin[0] * spacing[0],
        0.0, spacing[1], 0.0, -origin[1] * spacing[1],
        0.0, 0.0, spacing[2], -origin[2] * spacing[2],
        0.0, 0.0, 0.0, 1.0,
    )
}

pub fn set_affine(header: &mut NiftiHeader, affine4: &Matrix4<f64>) {
    // Set affine into sform with default code
    set_sform(header, affine4, 2);

    // Make qform 'unknown'
    set_qform(header, affine4, 0);
}

/// Set sform transform from 4x4 affine
pub fn set_sform(header: &mut NiftiHeader, affine4: &Matrix4<f64>, code: usize) {
    header.sform_code = code as i16;
    header.srow_x[0] = affine4[0] as f32;
    header.srow_x[1] = affine4[4] as f32;
    header.srow_x[2] = affine4[8] as f32;
    header.srow_x[3] = affine4[12] as f32;
    header.srow_y[0] = affine4[1] as f32;
    header.srow_y[1] = affine4[5] as f32;
    header.srow_y[2] = affine4[9] as f32;
    header.srow_y[3] = affine4[13] as f32;
    header.srow_z[0] = affine4[2] as f32;
    header.srow_z[1] = affine4[6] as f32;
    header.srow_z[2] = affine4[10] as f32;
    header.srow_z[3] = affine4[14] as f32;
}

/// Set qform header values from 4x4 affine
///
/// The qform transform only encodes translations, rotations and zooms. If there are shear
/// components to the `affine` transform, and `strip_shears` is True (the default), the written
/// qform gives the closest approximation where the rotation matrix is orthogonal. This is to allow
/// quaternion representation. The orthogonal representation enforces orthogonal axes.
pub fn set_qform(header: &mut NiftiHeader, affine4: &Matrix4<f64>, code: usize) {
    let (affine, translation) = get_affine_and_translation(&affine4);
    let aff2 = affine.component_mul(&affine);
    let spacing = (
        (aff2[0] + aff2[1] + aff2[2]).sqrt(),
        (aff2[3] + aff2[4] + aff2[5]).sqrt(),
        (aff2[6] + aff2[7] + aff2[8]).sqrt(),
    );
    let mut r = Matrix3::<f64>::new(
        affine[0] / spacing.0, affine[3] / spacing.1, affine[6] / spacing.2,
        affine[1] / spacing.0, affine[4] / spacing.1, affine[7] / spacing.2,
        affine[2] / spacing.0, affine[5] / spacing.1, affine[8] / spacing.2,
    );

    // Set qfac to make R determinant positive
    let qfac = if r.determinant() > 0.0 {
        1.0
    } else {
        r[6] *= -1.0;
        r[7] *= -1.0;
        r[8] *= -1.0;
        -1.0
    };

    // Make R orthogonal (to allow quaternion representation). The orthogonal representation
    // enforces orthogonal axes (a subtle requirement of the NIFTI format qform transform).
    // Transform below is polar decomposition, returning the closest orthogonal matrix PR, to input R.
    let svd = r.svd(true, true);
    let pr = svd.u.unwrap() * svd.v_t.unwrap();
    let quaternion = affine_to_quaternion(&pr);

    header.qform_code = code as i16;
    header.pixdim[0] = qfac;
    header.pixdim[1] = spacing.0 as f32;
    header.pixdim[2] = spacing.1 as f32;
    header.pixdim[3] = spacing.2 as f32;
    header.quatern_b = quaternion[1] as f32;
    header.quatern_c = quaternion[2] as f32;
    header.quatern_d = quaternion[3] as f32;
    header.quatern_x = translation[0] as f32;
    header.quatern_y = translation[1] as f32;
    header.quatern_z = translation[2] as f32;
}

/// Compute quaternion from b, c, d of quaternion
///
/// Fills a value by assuming this is a unit quaternion.
pub fn get_qform_quaternion(h: &NiftiHeader) -> Quaternion<f64> {
    fill_positive(
        Vector3::new(h.quatern_b as f64, h.quatern_c as f64, h.quatern_d as f64),
        Some(QUARTERNION_THRESHOLD),
    ) // TODO self.quaternion_threshold
}

/// Compute unit quaternion from last 3 values
///
/// If w, x, y, z are the values in the full quaternion, assumes w is positive.
/// Gives error if w*w is estimated to be negative.
/// w = 0 corresponds to a 180 degree rotation.
/// The unit quaternion specifies that np.dot(wxyz, wxyz) == 1.
///
/// If w is positive (assumed here), w is given by:
///     w = np.sqrt(1.0-(x*x+y*y+z*z))
/// w2 = 1.0-(x*x+y*y+z*z) can be near zero, which will lead to numerical instability in sqrt.
/// Here we use the system maximum float type to reduce numerical instability.
pub fn fill_positive(xyz: Vector3<f64>, w2_thresh: Option<f64>) -> Quaternion<f64> {
    let w2_thresh = if let Some(w2_thresh) = w2_thresh { w2_thresh } else { QUARTERNION_THRESHOLD };
    let w2 = 1.0 - xyz.dot(&xyz);
    let w = if w2 < 0.0 {
        if w2 < w2_thresh {
            panic!("w2 should be positive, but is {}", w2);
        }
        0.0
    } else {
        w2.sqrt()
    };
    Quaternion::new(w, xyz.x, xyz.y, xyz.z)
}

/// Calculate quaternion corresponding to given rotation matrix.
///
/// Method claimed to be robust to numerical errors in M. Constructs quaternion by calculating
/// maximum eigenvector for matrix K (constructed from input `M`).  Although this is not tested, a
/// maximum eigenvalue of 1 corresponds to a valid rotation.
///
/// A quaternion q*-1 corresponds to the same rotation as q; thus the sign of the reconstructed
/// quaternion is arbitrary, and we return quaternions with positive w (q[0]).
///
/// Bar-Itzhack, Itzhack Y. "New method for extracting the quaternion from a rotation
/// matrix", AIAA Journal of Guidance, Control and Dynamics 23(6):1085-1087, 2000
pub fn affine_to_quaternion(affine: &Matrix3<f64>) -> RowVector4<f64> {
    // Qyx refers to the contribution of the y input vector component to the x output vector
    // component. Qyx is therefore the same as M[0,1]. The notation is from the Wikipedia article.
    let qxx = affine[0];
    let qyx = affine[3];
    let qzx = affine[6];
    let qxy = affine[1];
    let qyy = affine[4];
    let qzy = affine[7];
    let qxz = affine[2];
    let qyz = affine[5];
    let qzz = affine[8];

    // Fill only lower half of symmetric matrix
    let k = Matrix4::new(
        qxx - qyy - qzz, 0.0,             0.0,             0.0,
        qyx + qxy,       qyy - qxx - qzz, 0.0,             0.0,
        qzx + qxz,       qzy + qyz,       qzz - qxx - qyy, 0.0,
        qyz - qzy,       qzx - qxz,       qxy - qyx,       qxx + qyy + qzz,
    );

    // Use Hermitian eigenvectors, values for speed
    let SymmetricEigen { eigenvalues: values, eigenvectors: vectors } = k.symmetric_eigen();

    // Select largest eigenvector, reorder to w,x,y,z quaternion
    let (max_idx, _) = values
        .as_slice()
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .unwrap();
    let max_vector = vectors.fixed_columns::<U1>(max_idx);
    let quaternion = RowVector4::new(max_vector[3], max_vector[0], max_vector[1], max_vector[2]);

    // Prefer quaternion with positive w
    // (q * -1 corresponds to same rotation as q)
    if quaternion[0] < 0.0 {
        quaternion * -1.0
    } else {
        quaternion
    }
}

/// Calculate rotation matrix corresponding to quaternion
///
/// Rotation matrix applies to column vectors, and is applied to the left of coordinate vectors.
/// The algorithm here allows non-unit quaternions.
///
/// Algorithm from https://en.wikipedia.org/wiki/Rotation_matrix#Quaternion
fn quaternion_to_affine(q: Quaternion<f64>) -> Matrix3<f64> {
    let nq = q.w * q.w + q.i * q.i + q.j * q.j + q.k * q.k;
    if nq < ::std::f64::EPSILON {
        return Matrix3::identity();
    }
    let s = 2.0 / nq;
    let x = q.i * s;
    let y = q.j * s;
    let z = q.k * s;
    let wx = q.w * x;
    let wy = q.w * y;
    let wz = q.w * z;
    let xx = q.i * x;
    let xy = q.i * y;
    let xz = q.i * z;
    let yy = q.j * y;
    let yz = q.j * z;
    let zz = q.k * z;
    Matrix3::new(
        1.0 - (yy + zz), xy - wz, xz + wy,
        xy + wz, 1.0 - (xx + zz), yz - wx,
        xz - wy, yz + wx, 1.0 - (xx + yy),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_zoom_affine() {
        let affine = shape_zoom_affine(&[3, 5, 7], &[3.0, 2.0, 1.0]);
        assert_eq!(affine, Affine4::new(
            -3.0, 0.0, 0.0, 3.0,
            0.0, 2.0, 0.0, -4.0,
            0.0, 0.0, 1.0, -3.0,
            0.0, 0.0, 0.0, 1.0,
        ));

        let affine = shape_zoom_affine(&[256, 256, 54], &[0.9375, 0.9375, 3.0]);
        assert_eq!(affine, Affine4::new(
            -0.9375, 0.0, 0.0, 119.53125,
            0.0, 0.9375, 0.0, -119.53125,
            0.0, 0.0, 3.0, -79.5,
            0.0, 0.0, 0.0, 1.0,
        ));
    }

    #[test]
    fn test_fill_positive() {
        let q = fill_positive(Vector3::new(0.0, 0.0, 0.0), None);
        assert_eq!(q, Quaternion::new(1.0, 0.0, 0.0, 0.0));

        let q = fill_positive(Vector3::new(1.0, 0.0, 0.0), None);
        assert_eq!(q, Quaternion::new(0.0, 1.0, 0.0, 0.0));
        assert_eq!(q.dot(&q), 1.0);
    }

    #[test]
    fn test_affine_to_quaternion() {
        let affine = Matrix3::<f64>::identity();
        assert_eq!(affine_to_quaternion(&affine), RowVector4::new(1.0, 0.0, 0.0, 0.0));

        let affine = Matrix3::from_diagonal(&Vector3::new(1.0, -1.0, -1.0));
        assert_eq!(affine_to_quaternion(&affine), RowVector4::new(0.0, 1.0, 0.0, 0.0));

        let affine = Matrix3::new(1.1, 0.1, 0.1, 0.2, 1.1, 0.5, 0.0, 0.0, 1.0);
        assert_eq!(
            affine_to_quaternion(&affine),
            RowVector4::new(
                0.9929998817020886,
                -0.1147422705153119,
                0.017766153114299042,
                0.02167510323267157
            )
        );
    }

    #[test]
    fn test_quaternion_to_affine() {
        // Identity quaternion
        let affine = quaternion_to_affine(Quaternion::new(1.0, 0.0, 0.0, 0.0));
        assert_eq!(affine, Matrix3::identity());

        // 180 degree rotation around axis 0
        let affine = quaternion_to_affine(Quaternion::new(0.0, 1.0, 0.0, 0.0));
        assert_eq!(affine, Matrix3::new(1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0));
    }
}
