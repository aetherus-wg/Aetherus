//! Two-dimensional point.

use crate::{core::Real, math::Vec2};
use nalgebra::Point2 as P2;
use serde_derive::{Deserialize, Serialize};
use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};
use std::fmt::Display;

/// Two-dimensional real-number point.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Point2 {
    /// Internal data.
    #[serde(flatten)]
    data: P2<Real>,
}

impl Point2 {
    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(x: Real, y: Real) -> Self {
        Self {
            data: P2::new(x, y),
        }
    }

    /// Access the first component.
    #[inline]
    #[must_use]
    pub fn x(&self) -> Real {
        self.data.x
    }

    /// Access the second component.
    #[inline]
    #[must_use]
    pub fn y(&self) -> Real {
        self.data.y
    }
}

impl From<P2<Real>> for Point2 {
    #[inline]
    #[must_use]
    fn from(v: P2<Real>) -> Self {
        Self { data: v }
    }
}

impl Neg for Point2 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn neg(self) -> Self {
        Self::from(-self.data)
    }
}

impl Add<Vec2> for Point2 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn add(self, rhs: Vec2) -> Self {
        Self::from(self.data + rhs.data())
    }
}

impl Sub<Vec2> for Point2 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn sub(self, rhs: Vec2) -> Self {
        Self::from(self.data - rhs.data())
    }
}

impl Mul<Real> for Point2 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn mul(self, rhs: Real) -> Self {
        Self::from(self.data * rhs)
    }
}

impl Div<Real> for Point2 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn div(self, rhs: Real) -> Self {
        Self::from(self.data / rhs)
    }
}

impl AddAssign<Vec2> for Point2 {
    #[inline]
    fn add_assign(&mut self, rhs: Vec2) {
        self.data += rhs.data();
    }
}

impl SubAssign<Vec2> for Point2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Vec2) {
        self.data -= rhs.data();
    }
}

impl MulAssign<Real> for Point2 {
    #[inline]
    fn mul_assign(&mut self, rhs: Real) {
        self.data *= rhs;
    }
}

impl DivAssign<Real> for Point2 {
    #[inline]
    fn div_assign(&mut self, rhs: Real) {
        self.data /= rhs;
    }
}

impl Index<usize> for Point2 {
    type Output = Real;

    #[inline]
    #[must_use]
    fn index(&self, i: usize) -> &Self::Output {
        match i {
            0 => &self.data.x,
            1 => &self.data.y,
            _ => panic!("Out of bounds index for two-dimensional point."),
        }
    }
}

impl IndexMut<usize> for Point2 {
    #[inline]
    #[must_use]
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        match i {
            0 => &mut self.data.x,
            1 => &mut self.data.y,
            _ => panic!("Out of bounds index for two-dimensional point."),
        }
    }
}

impl Display for Point2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_new() {
        let point = Point2::new(17.0, -4.0);

        assert_approx_eq!(point.x(), 17.0);
        assert_approx_eq!(point.y(), -4.0);
    }

    #[test]
    fn test_convert() {
        let point = Point2::from(P2::new(1.23, -4.56));

        assert_approx_eq!(point.x(), 1.23);
        assert_approx_eq!(point.y(), -4.56);
    }

    #[test]
    fn test_neg() {
        let point = Point2::new(1.0, -4.0);

        let ans = -point;

        assert_approx_eq!(ans.x(), -1.0);
        assert_approx_eq!(ans.y(), 4.0);
    }

    #[test]
    fn test_add() {
        let point = Point2::new(1.0, -4.0);
        let vec = Vec2::new(5.0, -7.0);

        let ans = point + vec;

        assert_approx_eq!(ans.x(), 6.0);
        assert_approx_eq!(ans.y(), -11.0);
    }

    #[test]
    fn test_sub() {
        let point = Point2::new(1.0, -4.0);
        let vec = Vec2::new(5.0, -7.0);

        let ans = point - vec;

        assert_approx_eq!(ans.x(), -4.0);
        assert_approx_eq!(ans.y(), 3.0);
    }

    #[test]
    fn test_mul() {
        let point = Point2::new(1.0, -4.0);

        let ans = point * -5.0;

        assert_approx_eq!(ans.x(), -5.0);
        assert_approx_eq!(ans.y(), 20.0);
    }

    #[test]
    fn test_div() {
        let point = Point2::new(1.0, -4.0);

        let ans = point / -5.0;

        assert_approx_eq!(ans.x(), -0.2);
        assert_approx_eq!(ans.y(), 0.8);
    }

    #[test]
    fn test_add_assign() {
        let mut point = Point2::new(1.0, -4.0);
        let vec = Vec2::new(5.0, -7.0);

        point += vec;

        assert_approx_eq!(point.x(), 6.0);
        assert_approx_eq!(point.y(), -11.0);
    }

    #[test]
    fn test_sub_assign() {
        let mut point = Point2::new(1.0, -4.0);
        let vec = Vec2::new(5.0, -7.0);

        point -= vec;

        assert_approx_eq!(point.x(), -4.0);
        assert_approx_eq!(point.y(), 3.0);
    }

    #[test]
    fn test_mul_assign() {
        let mut point = Point2::new(1.0, -4.0);

        point *= -5.0;

        assert_approx_eq!(point.x(), -5.0);
        assert_approx_eq!(point.y(), 20.0);
    }

    #[test]
    fn test_div_assign() {
        let mut point = Point2::new(1.0, -4.0);

        point /= -5.0;

        assert_approx_eq!(point.x(), -0.2);
        assert_approx_eq!(point.y(), 0.8);
    }

    #[test]
    fn test_index() {
        let point = Point2::new(1.0, -4.0);

        assert_approx_eq!(point[0], 1.0);
        assert_approx_eq!(point[1], -4.0);
    }

    #[test]
    fn test_index_mut() {
        let mut point = Point2::new(1.0, -4.0);

        point[0] *= 2.0;
        point[1] /= -2.0;

        assert_approx_eq!(point[0], 2.0);
        assert_approx_eq!(point[1], 2.0);
    }

    #[test]
    #[should_panic]
    fn test_index_mut_out_of_bounds() {
        let point = Point2::new(1.0, -4.0);

        let _ = point[2];
    }
}
