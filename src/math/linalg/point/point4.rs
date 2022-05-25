//! Four-dimensional point.

use crate::{
    clone,
    core::Real,
    math::{Point3, Vec4},
};
use nalgebra::Point4 as P4;
use serde_derive::{Deserialize, Serialize};
use std::fmt::Display;
use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

/// Four-dimensional real-number point.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Point4 {
    /// Internal data.
    data: P4<Real>,
}

impl Point4 {
    clone!(data: P4<Real>);

    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(x: Real, y: Real, z: Real, w: Real) -> Self {
        Self {
            data: P4::new(x, y, z, w),
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

    /// Access the third component.
    #[inline]
    #[must_use]
    pub fn z(&self) -> Real {
        self.data.z
    }

    /// Access the fourth component.
    #[inline]
    #[must_use]
    pub fn w(&self) -> Real {
        self.data.w
    }

    #[inline]
    #[must_use]
    pub fn xyz(&self) -> Point3 {
        self.data.xyz().into()
    }
}

impl From<P4<Real>> for Point4 {
    #[inline]
    #[must_use]
    fn from(v: P4<Real>) -> Self {
        Self { data: v }
    }
}

impl Neg for Point4 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn neg(self) -> Self {
        Self::from(-self.data)
    }
}

impl Add<Vec4> for Point4 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn add(self, rhs: Vec4) -> Self {
        Self::from(self.data + rhs.data())
    }
}

impl Sub<Vec4> for Point4 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn sub(self, rhs: Vec4) -> Self {
        Self::from(self.data - rhs.data())
    }
}

impl Mul<Real> for Point4 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn mul(self, rhs: Real) -> Self {
        Self::from(self.data * rhs)
    }
}

impl Div<Real> for Point4 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn div(self, rhs: Real) -> Self {
        Self::from(self.data / rhs)
    }
}

impl AddAssign<Vec4> for Point4 {
    #[inline]
    fn add_assign(&mut self, rhs: Vec4) {
        self.data += rhs.data();
    }
}

impl SubAssign<Vec4> for Point4 {
    #[inline]
    fn sub_assign(&mut self, rhs: Vec4) {
        self.data -= rhs.data();
    }
}

impl MulAssign<Real> for Point4 {
    #[inline]
    fn mul_assign(&mut self, rhs: Real) {
        self.data *= rhs;
    }
}

impl DivAssign<Real> for Point4 {
    #[inline]
    fn div_assign(&mut self, rhs: Real) {
        self.data /= rhs;
    }
}

impl Index<usize> for Point4 {
    type Output = Real;

    #[inline]
    #[must_use]
    fn index(&self, i: usize) -> &Self::Output {
        match i {
            0 => &self.data.x,
            1 => &self.data.y,
            2 => &self.data.z,
            3 => &self.data.w,
            _ => panic!("Out of bounds index for four-dimensional point."),
        }
    }
}

impl IndexMut<usize> for Point4 {
    #[inline]
    #[must_use]
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        match i {
            0 => &mut self.data.x,
            1 => &mut self.data.y,
            2 => &mut self.data.z,
            3 => &mut self.data.w,
            _ => panic!("Out of bounds index for four-dimensional point."),
        }
    }
}

impl Display for Point4 {
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
        let point = Point4::new(17.0, -4.0, 23.0, 4.0);

        assert_approx_eq!(point.x(), 17.0);
        assert_approx_eq!(point.y(), -4.0);
        assert_approx_eq!(point.z(), 23.0);
        assert_approx_eq!(point.w(), 4.0);
    }

    #[test]
    fn test_convert() {
        let point = Point4::from(P4::new(1.23, -4.56, 7.89, -0.12));

        assert_approx_eq!(point.x(), 1.23);
        assert_approx_eq!(point.y(), -4.56);
        assert_approx_eq!(point.z(), 7.89);
        assert_approx_eq!(point.w(), -0.12);
    }

    #[test]
    fn test_neg() {
        let point = Point4::new(1.0, -4.0, 12.0, -17.0);

        let ans = -point;

        assert_approx_eq!(ans.x(), -1.0);
        assert_approx_eq!(ans.y(), 4.0);
        assert_approx_eq!(ans.z(), -12.0);
        assert_approx_eq!(ans.w(), 17.0);
    }

    #[test]
    fn test_add() {
        let point = Point4::new(1.0, -4.0, 12.0, -17.0);
        let vec = Vec4::new(5.0, -7.0, -11.0, 23.0);

        let ans = point + vec;

        assert_approx_eq!(ans.x(), 6.0);
        assert_approx_eq!(ans.y(), -11.0);
        assert_approx_eq!(ans.z(), 1.0);
        assert_approx_eq!(ans.w(), 6.0);
    }

    #[test]
    fn test_sub() {
        let point = Point4::new(1.0, -4.0, 12.0, -17.0);
        let vec = Vec4::new(5.0, -7.0, -11.0, 23.0);

        let ans = point - vec;

        assert_approx_eq!(ans.x(), -4.0);
        assert_approx_eq!(ans.y(), 3.0);
        assert_approx_eq!(ans.z(), 23.0);
        assert_approx_eq!(ans.w(), -40.0);
    }

    #[test]
    fn test_mul() {
        let point = Point4::new(1.0, -4.0, 12.0, -17.0);

        let ans = point * -5.0;

        assert_approx_eq!(ans.x(), -5.0);
        assert_approx_eq!(ans.y(), 20.0);
        assert_approx_eq!(ans.z(), -60.0);
        assert_approx_eq!(ans.w(), 85.0);
    }

    #[test]
    fn test_div() {
        let point = Point4::new(1.0, -4.0, 12.0, -17.0);

        let ans = point / -5.0;

        assert_approx_eq!(ans.x(), -0.2);
        assert_approx_eq!(ans.y(), 0.8);
        assert_approx_eq!(ans.z(), -2.4);
        assert_approx_eq!(ans.w(), 17.0 / 5.0);
    }

    #[test]
    fn test_add_assign() {
        let mut point = Point4::new(1.0, -4.0, 12.0, -17.0);
        let vec = Vec4::new(5.0, -7.0, -11.0, 23.0);

        point += vec;

        assert_approx_eq!(point.x(), 6.0);
        assert_approx_eq!(point.y(), -11.0);
        assert_approx_eq!(point.z(), 1.0);
        assert_approx_eq!(point.w(), 6.0);
    }

    #[test]
    fn test_sub_assign() {
        let mut point = Point4::new(1.0, -4.0, 12.0, -17.0);
        let vec = Vec4::new(5.0, -7.0, -11.0, 23.0);

        point -= vec;

        assert_approx_eq!(point.x(), -4.0);
        assert_approx_eq!(point.y(), 3.0);
        assert_approx_eq!(point.z(), 23.0);
        assert_approx_eq!(point.w(), -40.0);
    }

    #[test]
    fn test_mul_assign() {
        let mut point = Point4::new(1.0, -4.0, 12.0, -17.0);

        point *= -5.0;

        assert_approx_eq!(point.x(), -5.0);
        assert_approx_eq!(point.y(), 20.0);
        assert_approx_eq!(point.z(), -60.0);
        assert_approx_eq!(point.w(), 85.0);
    }

    #[test]
    fn test_div_assign() {
        let mut point = Point4::new(1.0, -4.0, 12.0, -17.0);

        point /= -5.0;

        assert_approx_eq!(point.x(), -0.2);
        assert_approx_eq!(point.y(), 0.8);
        assert_approx_eq!(point.z(), -2.4);
        assert_approx_eq!(point.w(), 17.0 / 5.0);
    }

    #[test]
    fn test_index() {
        let point = Point4::new(1.0, -4.0, 12.0, -17.0);

        assert_approx_eq!(point[0], 1.0);
        assert_approx_eq!(point[1], -4.0);
        assert_approx_eq!(point[2], 12.0);
        assert_approx_eq!(point[3], -17.0);
    }

    #[test]
    fn test_index_mut() {
        let mut point = Point4::new(1.0, -4.0, 12.0, -17.0);

        point[0] *= 2.0;
        point[1] /= -2.0;
        point[2] -= 2.0;
        point[3] += 2.0;

        assert_approx_eq!(point[0], 2.0);
        assert_approx_eq!(point[1], 2.0);
        assert_approx_eq!(point[2], 10.0);
        assert_approx_eq!(point[3], -15.0);
    }

    #[test]
    #[should_panic]
    fn test_index_mut_out_of_bounds() {
        let point = Point4::new(1.0, -4.0, 12.0, -17.0);

        let _ = point[4];
    }
}
