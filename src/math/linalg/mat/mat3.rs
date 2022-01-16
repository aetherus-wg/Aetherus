//! Square third-order matrix.

use crate::{core::Real, math::Vec3};
use nalgebra::Matrix3;
use serde_derive::{Deserialize, Serialize};

/// Three-by-three real-number matrix.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Mat3 {
    /// Internal data.
    data: Matrix3<Real>,
}

impl Mat3 {
    /// Construct a new instance from component row vectors.
    #[inline]
    #[must_use]
    pub fn new_from_rows(row_x: &Vec3, row_y: &Vec3, row_z: &Vec3) -> Self {
        Self {
            data: Matrix3::new(
                row_x.x(),
                row_x.y(),
                row_x.z(),
                row_y.x(),
                row_y.y(),
                row_y.z(),
                row_z.x(),
                row_z.y(),
                row_z.z(),
            ),
        }
    }

    /// Construct a new instance from component column vectors.
    #[inline]
    #[must_use]
    pub fn new_from_cols(col_x: &Vec3, col_y: &Vec3, col_z: &Vec3) -> Self {
        Self {
            data: Matrix3::new(
                col_x.x(),
                col_y.x(),
                col_z.x(),
                col_x.y(),
                col_y.y(),
                col_z.y(),
                col_x.z(),
                col_y.z(),
                col_z.z(),
            ),
        }
    }

    /// Access the top-left component.
    #[inline]
    #[must_use]
    pub fn xx(&self) -> Real {
        self.data.m11
    }

    /// Access the top-middle component.
    #[inline]
    #[must_use]
    pub fn xy(&self) -> Real {
        self.data.m12
    }

    /// Access the top-right component.
    #[inline]
    #[must_use]
    pub fn xz(&self) -> Real {
        self.data.m13
    }

    /// Access the middle-left component.
    #[inline]
    #[must_use]
    pub fn yx(&self) -> Real {
        self.data.m21
    }

    /// Access the middle-middle component.
    #[inline]
    #[must_use]
    pub fn yy(&self) -> Real {
        self.data.m22
    }

    /// Access the middle-right component.
    #[inline]
    #[must_use]
    pub fn yz(&self) -> Real {
        self.data.m23
    }

    /// Access the bottom-left component.
    #[inline]
    #[must_use]
    pub fn zx(&self) -> Real {
        self.data.m31
    }

    /// Access the bottom-middle component.
    #[inline]
    #[must_use]
    pub fn zy(&self) -> Real {
        self.data.m32
    }

    /// Access the bottom-right component.
    #[inline]
    #[must_use]
    pub fn zz(&self) -> Real {
        self.data.m33
    }

    /// Calculate the determinant.
    #[inline]
    #[must_use]
    pub fn det(&self) -> Real {
        self.data.determinant()
    }
}

impl From<Matrix3<Real>> for Mat3 {
    #[inline]
    #[must_use]
    fn from(d: Matrix3<Real>) -> Self {
        Self { data: d }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_new_from_rows() {
        let mat = Mat3::new_from_rows(
            &Vec3::new(2.0, -3.0, 1.0),
            &Vec3::new(2.0, 0.0, -1.0),
            &Vec3::new(1.0, 4.0, 5.0),
        );

        assert_approx_eq!(mat.xx(), 2.0);
        assert_approx_eq!(mat.xy(), -3.0);
        assert_approx_eq!(mat.xz(), 1.0);
        assert_approx_eq!(mat.yx(), 2.0);
        assert_approx_eq!(mat.yy(), 0.0);
        assert_approx_eq!(mat.yz(), -1.0);
        assert_approx_eq!(mat.zx(), 1.0);
        assert_approx_eq!(mat.zy(), 4.0);
        assert_approx_eq!(mat.zz(), 5.0);
    }

    #[test]
    fn test_new_from_cols() {
        let mat = Mat3::new_from_cols(
            &Vec3::new(2.0, -3.0, 1.0),
            &Vec3::new(2.0, 0.0, -1.0),
            &Vec3::new(1.0, 4.0, 5.0),
        );

        assert_approx_eq!(mat.xx(), 2.0);
        assert_approx_eq!(mat.xy(), 2.0);
        assert_approx_eq!(mat.xz(), 1.0);
        assert_approx_eq!(mat.yx(), -3.0);
        assert_approx_eq!(mat.yy(), 0.0);
        assert_approx_eq!(mat.yz(), 4.0);
        assert_approx_eq!(mat.zx(), 1.0);
        assert_approx_eq!(mat.zy(), -1.0);
        assert_approx_eq!(mat.zz(), 5.0);
    }

    #[test]
    fn test_det() {
        let mat = Mat3::new_from_rows(
            &Vec3::new(2.0, -3.0, 1.0),
            &Vec3::new(2.0, 0.0, -1.0),
            &Vec3::new(1.0, 4.0, 5.0),
        );

        assert_approx_eq!(mat.det(), 49.0);
    }
}
