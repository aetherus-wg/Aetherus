//! Square second-order matrix.

use crate::{core::Real, math::Vec2};
use nalgebra::Matrix2;

/// Two-by-two real-number matrix.
pub struct Mat2 {
    /// Internal data.
    data: Matrix2<f64>,
}

impl Mat2 {
    /// Construct a new instance from component row vectors.
    #[inline]
    #[must_use]
    pub fn new_from_rows(row_x: &Vec2, row_y: &Vec2) -> Self {
        Self {
            data: Matrix2::new(row_x.x(), row_x.y(), row_y.x(), row_y.y()),
        }
    }

    /// Construct a new instance from component column vectors.
    #[inline]
    #[must_use]
    pub fn new_from_cols(col_x: &Vec2, col_y: &Vec2) -> Self {
        Self {
            data: Matrix2::new(col_x.x(), col_y.x(), col_x.y(), col_y.y()),
        }
    }

    /// Access the top-left component.
    #[inline]
    #[must_use]
    pub fn xx(&self) -> Real {
        self.data.m11
    }

    /// Access the top-right component.
    #[inline]
    #[must_use]
    pub fn xy(&self) -> Real {
        self.data.m12
    }

    /// Access the bottom-left component.
    #[inline]
    #[must_use]
    pub fn yx(&self) -> Real {
        self.data.m21
    }

    /// Access the bottom-right component.
    #[inline]
    #[must_use]
    pub fn yy(&self) -> Real {
        self.data.m22
    }

    /// Calculate the determinant.
    #[inline]
    #[must_use]
    pub fn det(&self) -> Real {
        self.data.determinant()
    }
}

impl From<Matrix2<Real>> for Mat2 {
    #[inline]
    #[must_use]
    fn from(d: Matrix2<Real>) -> Self {
        Self { data: d }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_new_from_rows() {
        let mat = Mat2::new_from_rows(&Vec2::new(3.0, 8.0), &Vec2::new(4.0, 6.0));

        assert_approx_eq!(mat.xx(), 3.0);
        assert_approx_eq!(mat.xy(), 8.0);
        assert_approx_eq!(mat.yx(), 4.0);
        assert_approx_eq!(mat.yy(), 6.0);
    }

    #[test]
    fn test_new_from_cols() {
        let mat = Mat2::new_from_cols(&Vec2::new(3.0, 8.0), &Vec2::new(4.0, 6.0));

        assert_approx_eq!(mat.xx(), 3.0);
        assert_approx_eq!(mat.xy(), 4.0);
        assert_approx_eq!(mat.yx(), 8.0);
        assert_approx_eq!(mat.yy(), 6.0);
    }

    #[test]
    fn test_det() {
        let mat = Mat2::new_from_rows(&Vec2::new(3.0, 8.0), &Vec2::new(4.0, 6.0));

        assert_approx_eq!(mat.det(), -14.0);
    }
}
