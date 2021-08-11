//! Three-dimensional point.

use crate::{core::Real, math::Vec3};
use nalgebra::Point3 as P3;
use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

/// Three-dimensional real-number point.
pub struct Point3 {
    /// Internal data.
    data: P3<Real>,
}

impl Point3 {
    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(x: Real, y: Real, z: Real) -> Self {
        Self {
            data: P3::new(x, y, z),
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
}

impl From<P3<Real>> for Point3 {
    #[inline]
    #[must_use]
    fn from(v: P3<Real>) -> Self {
        Self { data: v }
    }
}

impl Neg for Point3 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn neg(self) -> Self {
        Self::from(-self.data)
    }
}

impl Add<Vec3> for Point3 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn add(self, rhs: Vec3) -> Self {
        Self::from(self.data + rhs.data())
    }
}

impl Sub<Vec3> for Point3 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn sub(self, rhs: Vec3) -> Self {
        Self::from(self.data - rhs.data())
    }
}

impl Mul<Real> for Point3 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn mul(self, rhs: Real) -> Self {
        Self::from(self.data * rhs)
    }
}

impl Div<Real> for Point3 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn div(self, rhs: Real) -> Self {
        Self::from(self.data / rhs)
    }
}

impl AddAssign<Vec3> for Point3 {
    #[inline]
    fn add_assign(&mut self, rhs: Vec3) {
        self.data += rhs.data();
    }
}

impl SubAssign<Vec3> for Point3 {
    #[inline]
    fn sub_assign(&mut self, rhs: Vec3) {
        self.data -= rhs.data();
    }
}

impl MulAssign<Real> for Point3 {
    #[inline]
    fn mul_assign(&mut self, rhs: Real) {
        self.data *= rhs;
    }
}

impl DivAssign<Real> for Point3 {
    #[inline]
    fn div_assign(&mut self, rhs: Real) {
        self.data /= rhs;
    }
}

impl Index<usize> for Point3 {
    type Output = Real;

    #[inline]
    #[must_use]
    fn index(&self, i: usize) -> &Self::Output {
        match i {
            0 => &self.data.x,
            1 => &self.data.y,
            2 => &self.data.z,
            _ => panic!("Out of bounds index for three-dimensional point."),
        }
    }
}

impl IndexMut<usize> for Point3 {
    #[inline]
    #[must_use]
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        match i {
            0 => &mut self.data.x,
            1 => &mut self.data.y,
            2 => &mut self.data.z,
            _ => panic!("Out of bounds index for three-dimensional point."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_new() {
        let point = Point3::new(17.0, -4.0, 23.0);

        assert_approx_eq!(point.x(), 17.0);
        assert_approx_eq!(point.y(), -4.0);
        assert_approx_eq!(point.z(), 23.0);
    }

    #[test]
    fn test_convert() {
        let point = Point3::from(P3::new(1.23, -4.56, 7.89));

        assert_approx_eq!(point.x(), 1.23);
        assert_approx_eq!(point.y(), -4.56);
        assert_approx_eq!(point.z(), 7.89);
    }

    #[test]
    fn test_neg() {
        let point = Point3::new(1.0, -4.0, 12.0);

        let ans = -point;

        assert_approx_eq!(ans.x(), -1.0);
        assert_approx_eq!(ans.y(), 4.0);
        assert_approx_eq!(ans.z(), -12.0);
    }

    #[test]
    fn test_add() {
        let point = Point3::new(1.0, -4.0, 12.0);
        let vec = Vec3::new(5.0, -7.0, -11.0);

        let ans = point + vec;

        assert_approx_eq!(ans.x(), 6.0);
        assert_approx_eq!(ans.y(), -11.0);
        assert_approx_eq!(ans.z(), 1.0);
    }

    #[test]
    fn test_sub() {
        let point = Point3::new(1.0, -4.0, 12.0);
        let vec = Vec3::new(5.0, -7.0, -11.0);

        let ans = point - vec;

        assert_approx_eq!(ans.x(), -4.0);
        assert_approx_eq!(ans.y(), 3.0);
        assert_approx_eq!(ans.z(), 23.0);
    }

    #[test]
    fn test_mul() {
        let point = Point3::new(1.0, -4.0, 12.0);

        let ans = point * -5.0;

        assert_approx_eq!(ans.x(), -5.0);
        assert_approx_eq!(ans.y(), 20.0);
        assert_approx_eq!(ans.z(), -60.0);
    }

    #[test]
    fn test_div() {
        let point = Point3::new(1.0, -4.0, 12.0);

        let ans = point / -5.0;

        assert_approx_eq!(ans.x(), -0.2);
        assert_approx_eq!(ans.y(), 0.8);
        assert_approx_eq!(ans.z(), -2.4);
    }

    #[test]
    fn test_add_assign() {
        let mut point = Point3::new(1.0, -4.0, 12.0);
        let vec = Vec3::new(5.0, -7.0, -11.0);

        point += vec;

        assert_approx_eq!(point.x(), 6.0);
        assert_approx_eq!(point.y(), -11.0);
        assert_approx_eq!(point.z(), 1.0);
    }

    #[test]
    fn test_sub_assign() {
        let mut point = Point3::new(1.0, -4.0, 12.0);
        let vec = Vec3::new(5.0, -7.0, -11.0);

        point -= vec;

        assert_approx_eq!(point.x(), -4.0);
        assert_approx_eq!(point.y(), 3.0);
        assert_approx_eq!(point.z(), 23.0);
    }

    #[test]
    fn test_mul_assign() {
        let mut point = Point3::new(1.0, -4.0, 12.0);

        point *= -5.0;

        assert_approx_eq!(point.x(), -5.0);
        assert_approx_eq!(point.y(), 20.0);
        assert_approx_eq!(point.z(), -60.0);
    }

    #[test]
    fn test_div_assign() {
        let mut point = Point3::new(1.0, -4.0, 12.0);

        point /= -5.0;

        assert_approx_eq!(point.x(), -0.2);
        assert_approx_eq!(point.y(), 0.8);
        assert_approx_eq!(point.z(), -2.4);
    }

    #[test]
    fn test_index() {
        let point = Point3::new(1.0, -4.0, 12.0);

        assert_approx_eq!(point[0], 1.0);
        assert_approx_eq!(point[1], -4.0);
        assert_approx_eq!(point[2], 12.0);
    }

    #[test]
    fn test_index_mut() {
        let mut point = Point3::new(1.0, -4.0, 12.0);

        point[0] *= 2.0;
        point[1] /= -2.0;
        point[2] -= 2.0;

        assert_approx_eq!(point[0], 2.0);
        assert_approx_eq!(point[1], 2.0);
        assert_approx_eq!(point[2], 10.0);
    }

    #[test]
    #[should_panic]
    fn test_index_mut_out_of_bounds() {
        let point = Point3::new(1.0, -4.0, 12.0);

        let _ = point[3];
    }
}
