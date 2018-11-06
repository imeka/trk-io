use nalgebra::{Matrix3, Quaternion, Vector3};
use nifti::NiftiHeader;

use Affine4;

pub fn raw_affine_from_nifti(h: &NiftiHeader) -> Affine4 {
    if h.sform_code != 0 {
        get_sform(h)
    } else {
        get_qform(h)
    }
    // TODO else return base_affine?
}

pub fn get_sform(h: &NiftiHeader) -> Affine4 {
    Affine4::new(h.srow_x[0], h.srow_x[1], h.srow_x[2], h.srow_x[3],
                 h.srow_y[0], h.srow_y[1], h.srow_y[2], h.srow_y[3],
                 h.srow_z[0], h.srow_z[1], h.srow_z[2], h.srow_z[3],
                 0.0, 0.0, 0.0, 1.0)
}

/// Return 4x4 affine matrix from qform parameters in header
pub fn get_qform(h: &NiftiHeader) -> Affine4 {
    if h.pixdim[1] < 0.0 || h.pixdim[2] < 0.0 || h.pixdim[3] < 0.0 {
        panic!("All spacings (pixdim) should be positive");
    }
    if h.pixdim[0].abs() != 1.0 {
        panic!("qfac (pixdim[0]) should be 1 or -1");
    }

    let quaternion = get_qform_quaternion(h);
    let r = quaternion_to_affine(quaternion);
    let s = Matrix3::from_diagonal(&Vector3::new(
        h.pixdim[1] as f64,
        h.pixdim[2] as f64,
        (h.pixdim[3] * h.pixdim[0]) as f64));
    let m = r * s;
    Affine4::new(
        m[0] as f32, m[3] as f32, m[6] as f32, h.quatern_x,
        m[1] as f32, m[4] as f32, m[7] as f32, h.quatern_y,
        m[2] as f32, m[5] as f32, m[8] as f32, h.quatern_z,
        0.0, 0.0, 0.0, 1.0)
}

/// Compute quaternion from b, c, d of quaternion
///
/// Fills a value by assuming this is a unit quaternion.
pub fn get_qform_quaternion(h: &NiftiHeader) -> Quaternion<f64> {
    fill_positive(
        Vector3::new(h.quatern_b as f64, h.quatern_c as f64, h.quatern_d as f64),
        Some(-3.5762786865234375e-07)) // TODO self.quaternion_threshold
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
    let w2_thresh = if let Some(w2_thresh) = w2_thresh {
        w2_thresh
    } else {
        ::std::f64::EPSILON * 3.0
    };
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
        xz - wy, yz + wx, 1.0 - (xx + yy))
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_fill_positive() {
        let q = fill_positive(Vector3::new(0.0, 0.0, 0.0), None);
        assert_eq!(q, Quaternion::new(1.0, 0.0, 0.0, 0.0));

        let q = fill_positive(Vector3::new(1.0, 0.0, 0.0), None);
        assert_eq!(q, Quaternion::new(0.0, 1.0, 0.0, 0.0));
        assert_eq!(q.dot(&q), 1.0);
    }

    #[test]
    fn test_quaternion_to_affine() {
        // Identity quaternion
        let affine = quaternion_to_affine(Quaternion::new(1.0, 0.0, 0.0, 0.0));
        assert_eq!(affine, Matrix3::identity());

        // 180 degree rotation around axis 0
        let affine = quaternion_to_affine(Quaternion::new(0.0, 1.0, 0.0, 0.0));
        assert_eq!(affine, Matrix3::new(1.0, 0.0, 0.0,
                                        0.0, -1.0, 0.0,
                                        0.0, 0.0, -1.0));
    }
}
