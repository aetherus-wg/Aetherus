//! Square fourth-order matrix.

use crate::{
    clone,
    core::Real,
    math::{Dir3, Point3, Vec4},
};
use nalgebra::Matrix4;
use serde_derive::{Deserialize, Serialize};
use std::ops::Mul;

/// Four-by-four real-number matrix.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Mat4 {
    /// Internal data.
    data: Matrix4<Real>,
}

impl Mat4 {
    clone!(data: Matrix4<Real>);

    /// Construct a new instance from component row vectors.
    #[inline]
    #[must_use]
    pub fn new_from_rows(row_x: &Vec4, row_y: &Vec4, row_z: &Vec4, row_w: &Vec4) -> Self {
        Self {
            data: Matrix4::new(
                row_x.x(),
                row_x.y(),
                row_x.z(),
                row_x.w(),
                row_y.x(),
                row_y.y(),
                row_y.z(),
                row_y.w(),
                row_z.x(),
                row_z.y(),
                row_z.z(),
                row_z.w(),
                row_w.x(),
                row_w.y(),
                row_w.z(),
                row_w.w(),
            ),
        }
    }

    /// Construct a new instance from component column vectors.
    #[inline]
    #[must_use]
    pub fn new_from_cols(col_x: &Vec4, col_y: &Vec4, col_z: &Vec4, col_w: &Vec4) -> Self {
        Self {
            data: Matrix4::new(
                col_x.x(),
                col_y.x(),
                col_z.x(),
                col_w.x(),
                col_x.y(),
                col_y.y(),
                col_z.y(),
                col_w.y(),
                col_x.z(),
                col_y.z(),
                col_z.z(),
                col_w.z(),
                col_x.w(),
                col_y.w(),
                col_z.w(),
                col_w.w(),
            ),
        }
    }

    /// A function that builds a right-handed look at view matrix.
    pub fn look_at_rh(eye: &Point3, target: &Point3, up: &Dir3) -> Self {
        Self {
            data: nalgebra::Matrix4::look_at_rh(&eye.data(), &target.data(), &up.data()),
        }
    }

    /// Builds a new homogeneous matrix for orthographic projection.
    pub fn new_perspective(aspect_ratio: Real, fovy: Real, znear: Real, zfar: Real) -> Self {
        Self {
            data: nalgebra::Matrix4::new_perspective(aspect_ratio, fovy, znear, zfar),
        }
    }

    /// Access the top-left component.
    #[inline]
    #[must_use]
    pub fn xx(&self) -> Real {
        self.data.m11
    }

    /// Access the top-middle-left component.
    #[inline]
    #[must_use]
    pub fn xy(&self) -> Real {
        self.data.m12
    }

    /// Access the top-middle-right component.
    #[inline]
    #[must_use]
    pub fn xz(&self) -> Real {
        self.data.m13
    }

    /// Access the top-right component.
    #[inline]
    #[must_use]
    pub fn xw(&self) -> Real {
        self.data.m14
    }

    /// Access the middle-top-left component.
    #[inline]
    #[must_use]
    pub fn yx(&self) -> Real {
        self.data.m21
    }

    /// Access the middle-top-middle-left component.
    #[inline]
    #[must_use]
    pub fn yy(&self) -> Real {
        self.data.m22
    }

    /// Access the middle-top-middle-right component.
    #[inline]
    #[must_use]
    pub fn yz(&self) -> Real {
        self.data.m23
    }

    /// Access the middle-top-right component.
    #[inline]
    #[must_use]
    pub fn yw(&self) -> Real {
        self.data.m24
    }

    /// Access the middle-bottom-left component.
    #[inline]
    #[must_use]
    pub fn zx(&self) -> Real {
        self.data.m31
    }

    /// Access the middle-bottom-middle-left component.
    #[inline]
    #[must_use]
    pub fn zy(&self) -> Real {
        self.data.m32
    }

    /// Access the middle-bottom-middle-right component.
    #[inline]
    #[must_use]
    pub fn zz(&self) -> Real {
        self.data.m33
    }

    /// Access the middle-bottom-right component.
    #[inline]
    #[must_use]
    pub fn zw(&self) -> Real {
        self.data.m34
    }

    /// Access the bottom-left component.
    #[inline]
    #[must_use]
    pub fn wx(&self) -> Real {
        self.data.m41
    }

    /// Access the bottom-middle-left component.
    #[inline]
    #[must_use]
    pub fn wy(&self) -> Real {
        self.data.m42
    }

    /// Access the bottom-middle-right component.
    #[inline]
    #[must_use]
    pub fn wz(&self) -> Real {
        self.data.m43
    }

    /// Access the bottom-right component.
    #[inline]
    #[must_use]
    pub fn ww(&self) -> Real {
        self.data.m44
    }

    /// Calculate the determinant.
    #[inline]
    #[must_use]
    pub fn det(&self) -> Real {
        self.data.determinant()
    }
}

impl From<Matrix4<Real>> for Mat4 {
    #[inline]
    fn from(d: Matrix4<Real>) -> Self {
        Self { data: d }
    }
}

impl Mul<Mat4> for Mat4 {
    type Output = Mat4;
    fn mul(self, rhs: Mat4) -> Self::Output {
        Self {
            data: self.data * rhs.data,
        }
    }
}

impl Mul<Vec4> for Mat4 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Self::Output {
        Vec4::from(self.data * rhs.data())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_new_from_rows() {
        let mat = Mat4::new_from_rows(
            &Vec4::new(2.0, 5.0, 1.0, -3.0),
            &Vec4::new(-4.0, 1.0, 7.0, 9.0),
            &Vec4::new(6.0, 8.0, -3.0, 2.0),
            &Vec4::new(7.0, -8.0, 1.0, 4.0),
        );

        assert_approx_eq!(mat.xx(), 2.0);
        assert_approx_eq!(mat.xy(), 5.0);
        assert_approx_eq!(mat.xz(), 1.0);
        assert_approx_eq!(mat.xw(), -3.0);
        assert_approx_eq!(mat.yx(), -4.0);
        assert_approx_eq!(mat.yy(), 1.0);
        assert_approx_eq!(mat.yz(), 7.0);
        assert_approx_eq!(mat.yw(), 9.0);
        assert_approx_eq!(mat.zx(), 6.0);
        assert_approx_eq!(mat.zy(), 8.0);
        assert_approx_eq!(mat.zz(), -3.0);
        assert_approx_eq!(mat.zw(), 2.0);
        assert_approx_eq!(mat.wx(), 7.0);
        assert_approx_eq!(mat.wy(), -8.0);
        assert_approx_eq!(mat.wz(), 1.0);
        assert_approx_eq!(mat.ww(), 4.0);
    }

    #[test]
    fn test_new_from_cols() {
        let mat = Mat4::new_from_cols(
            &Vec4::new(2.0, 5.0, 1.0, -3.0),
            &Vec4::new(-4.0, 1.0, 7.0, 9.0),
            &Vec4::new(6.0, 8.0, -3.0, 2.0),
            &Vec4::new(7.0, -8.0, 1.0, 4.0),
        );

        assert_approx_eq!(mat.xx(), 2.0);
        assert_approx_eq!(mat.xy(), -4.0);
        assert_approx_eq!(mat.xz(), 6.0);
        assert_approx_eq!(mat.xw(), 7.0);
        assert_approx_eq!(mat.yx(), 5.0);
        assert_approx_eq!(mat.yy(), 1.0);
        assert_approx_eq!(mat.yz(), 8.0);
        assert_approx_eq!(mat.yw(), -8.0);
        assert_approx_eq!(mat.zx(), 1.0);
        assert_approx_eq!(mat.zy(), 7.0);
        assert_approx_eq!(mat.zz(), -3.0);
        assert_approx_eq!(mat.zw(), 1.0);
        assert_approx_eq!(mat.wx(), -3.0);
        assert_approx_eq!(mat.wy(), 9.0);
        assert_approx_eq!(mat.wz(), 2.0);
        assert_approx_eq!(mat.ww(), 4.0);
    }

    #[test]
    fn test_det() {
        let mat = Mat4::new_from_rows(
            &Vec4::new(2.0, 5.0, 1.0, -3.0),
            &Vec4::new(-4.0, 1.0, 7.0, 9.0),
            &Vec4::new(6.0, 8.0, -3.0, 2.0),
            &Vec4::new(7.0, -8.0, 1.0, 4.0),
        );

        assert_approx_eq!(mat.det(), -5344.0);
    }
}
