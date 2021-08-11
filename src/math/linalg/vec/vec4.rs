//! Four-dimensional vector.

use crate::{clone, core::Real, math::Dir4};
use nalgebra::{Unit, Vector4};
use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

/// Four-dimensional real-number vector.
pub struct Vec4 {
    /// Internal data.
    data: Vector4<Real>,
}

impl Vec4 {
    clone!(data: Vector4<Real>);

    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub const fn new(x: Real, y: Real, z: Real, w: Real) -> Self {
        Self {
            data: Vector4::new(x, y, z, w),
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

    /// Calculate the magnitude of the vector.
    #[inline]
    #[must_use]
    pub fn mag(&self) -> Real {
        self.data.magnitude()
    }

    /// Calculate the unit vector.
    #[inline]
    #[must_use]
    pub fn dir(&self) -> Dir4 {
        Dir4::from(Unit::new_normalize(self.data))
    }
}

impl From<Vector4<Real>> for Vec4 {
    #[inline]
    #[must_use]
    fn from(v: Vector4<Real>) -> Self {
        Self { data: v }
    }
}

impl Neg for Vec4 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn neg(self) -> Self {
        Self::from(-self.data)
    }
}

impl Add for Vec4 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn add(self, rhs: Self) -> Self {
        Self::from(self.data + rhs.data)
    }
}

impl Sub for Vec4 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn sub(self, rhs: Self) -> Self {
        Self::from(self.data - rhs.data)
    }
}

impl Mul<Real> for Vec4 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn mul(self, rhs: Real) -> Self {
        Self::from(self.data * rhs)
    }
}

impl Div<Real> for Vec4 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn div(self, rhs: Real) -> Self {
        Self::from(self.data / rhs)
    }
}

impl AddAssign<Self> for Vec4 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.data += rhs.data;
    }
}

impl SubAssign<Self> for Vec4 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.data -= rhs.data;
    }
}

impl MulAssign<Real> for Vec4 {
    #[inline]
    fn mul_assign(&mut self, rhs: Real) {
        self.data *= rhs;
    }
}

impl DivAssign<Real> for Vec4 {
    #[inline]
    fn div_assign(&mut self, rhs: Real) {
        self.data /= rhs;
    }
}

impl Mul for Vec4 {
    type Output = Real;

    #[inline]
    #[must_use]
    fn mul(self, rhs: Self) -> Self::Output {
        self.data.dot(&rhs.data)
    }
}

impl Index<usize> for Vec4 {
    type Output = Real;

    #[inline]
    #[must_use]
    fn index(&self, i: usize) -> &Self::Output {
        match i {
            0 => &self.data.x,
            1 => &self.data.y,
            2 => &self.data.z,
            3 => &self.data.w,
            _ => panic!("Out of bounds index for four-dimensional vector."),
        }
    }
}

impl IndexMut<usize> for Vec4 {
    #[inline]
    #[must_use]
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        match i {
            0 => &mut self.data.x,
            1 => &mut self.data.y,
            2 => &mut self.data.z,
            3 => &mut self.data.w,
            _ => panic!("Out of bounds index for four-dimensional vector."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_new() {
        let vec = Vec4::new(17.0, -4.0, 23.0, 4.0);

        assert_approx_eq!(vec.x(), 17.0);
        assert_approx_eq!(vec.y(), -4.0);
        assert_approx_eq!(vec.z(), 23.0);
        assert_approx_eq!(vec.w(), 4.0);
    }

    #[test]
    fn test_mag() {
        let vec = Vec4::new(2.0, -13.0, 14.0, -16.0);

        assert_approx_eq!(vec.mag(), 25.0);
    }

    #[test]
    fn test_dir() {
        let vec = Vec4::new(2.0, -13.0, 14.0, -16.0);

        let dir = vec.dir();

        assert_approx_eq!(dir.x(), 2.0 / 25.0);
        assert_approx_eq!(dir.y(), -13.0 / 25.0);
        assert_approx_eq!(dir.z(), 14.0 / 25.0);
        assert_approx_eq!(dir.w(), -16.0 / 25.0);
    }

    #[test]
    fn test_convert() {
        let vec = Vec4::from(Vector4::new(1.23, -4.56, 7.89, -0.12));

        assert_approx_eq!(vec.x(), 1.23);
        assert_approx_eq!(vec.y(), -4.56);
        assert_approx_eq!(vec.z(), 7.89);
        assert_approx_eq!(vec.w(), -0.12);
    }

    #[test]
    fn test_neg() {
        let vec = Vec4::new(1.0, -4.0, 12.0, -17.0);

        let ans = -vec;

        assert_approx_eq!(ans.x(), -1.0);
        assert_approx_eq!(ans.y(), 4.0);
        assert_approx_eq!(ans.z(), -12.0);
        assert_approx_eq!(ans.w(), 17.0);
    }

    #[test]
    fn test_add() {
        let vec_a = Vec4::new(1.0, -4.0, 12.0, -17.0);
        let vec_b = Vec4::new(5.0, -7.0, -11.0, 23.0);

        let ans = vec_a + vec_b;

        assert_approx_eq!(ans.x(), 6.0);
        assert_approx_eq!(ans.y(), -11.0);
        assert_approx_eq!(ans.z(), 1.0);
        assert_approx_eq!(ans.w(), 6.0);
    }

    #[test]
    fn test_sub() {
        let vec_a = Vec4::new(1.0, -4.0, 12.0, -17.0);
        let vec_b = Vec4::new(5.0, -7.0, -11.0, 23.0);

        let ans = vec_a - vec_b;

        assert_approx_eq!(ans.x(), -4.0);
        assert_approx_eq!(ans.y(), 3.0);
        assert_approx_eq!(ans.z(), 23.0);
        assert_approx_eq!(ans.w(), -40.0);
    }

    #[test]
    fn test_mul() {
        let vec = Vec4::new(1.0, -4.0, 12.0, -17.0);

        let ans = vec * -5.0;

        assert_approx_eq!(ans.x(), -5.0);
        assert_approx_eq!(ans.y(), 20.0);
        assert_approx_eq!(ans.z(), -60.0);
        assert_approx_eq!(ans.w(), 85.0);
    }

    #[test]
    fn test_div() {
        let vec = Vec4::new(1.0, -4.0, 12.0, -17.0);

        let ans = vec / -5.0;

        assert_approx_eq!(ans.x(), -0.2);
        assert_approx_eq!(ans.y(), 0.8);
        assert_approx_eq!(ans.z(), -2.4);
        assert_approx_eq!(ans.w(), 17.0 / 5.0);
    }

    #[test]
    fn test_add_assign() {
        let mut vec_a = Vec4::new(1.0, -4.0, 12.0, -17.0);
        let vec_b = Vec4::new(5.0, -7.0, -11.0, 23.0);

        vec_a += vec_b;

        assert_approx_eq!(vec_a.x(), 6.0);
        assert_approx_eq!(vec_a.y(), -11.0);
        assert_approx_eq!(vec_a.z(), 1.0);
        assert_approx_eq!(vec_a.w(), 6.0);
    }

    #[test]
    fn test_sub_assign() {
        let mut vec_a = Vec4::new(1.0, -4.0, 12.0, -17.0);
        let vec_b = Vec4::new(5.0, -7.0, -11.0, 23.0);

        vec_a -= vec_b;

        assert_approx_eq!(vec_a.x(), -4.0);
        assert_approx_eq!(vec_a.y(), 3.0);
        assert_approx_eq!(vec_a.z(), 23.0);
        assert_approx_eq!(vec_a.w(), -40.0);
    }

    #[test]
    fn test_mul_assign() {
        let mut vec = Vec4::new(1.0, -4.0, 12.0, -17.0);

        vec *= -5.0;

        assert_approx_eq!(vec.x(), -5.0);
        assert_approx_eq!(vec.y(), 20.0);
        assert_approx_eq!(vec.z(), -60.0);
        assert_approx_eq!(vec.w(), 85.0);
    }

    #[test]
    fn test_div_assign() {
        let mut vec = Vec4::new(1.0, -4.0, 12.0, -17.0);

        vec /= -5.0;

        assert_approx_eq!(vec.x(), -0.2);
        assert_approx_eq!(vec.y(), 0.8);
        assert_approx_eq!(vec.z(), -2.4);
        assert_approx_eq!(vec.w(), 17.0 / 5.0);
    }

    #[test]
    fn test_dot_prod() {
        let vec_a = Vec4::new(1.0, -4.0, 12.0, -17.0);
        let vec_b = Vec4::new(5.0, -7.0, -11.0, 23.0);

        let ans = vec_a * vec_b;

        assert_approx_eq!(ans, 5.0 + 28.0 + -132.0 + -391.0);
    }

    #[test]
    fn test_index() {
        let vec = Vec4::new(1.0, -4.0, 12.0, -17.0);

        assert_approx_eq!(vec[0], 1.0);
        assert_approx_eq!(vec[1], -4.0);
        assert_approx_eq!(vec[2], 12.0);
        assert_approx_eq!(vec[3], -17.0);
    }

    #[test]
    fn test_index_mut() {
        let mut vec = Vec4::new(1.0, -4.0, 12.0, -17.0);

        vec[0] *= 2.0;
        vec[1] /= -2.0;
        vec[2] -= 2.0;
        vec[3] += 2.0;

        assert_approx_eq!(vec[0], 2.0);
        assert_approx_eq!(vec[1], 2.0);
        assert_approx_eq!(vec[2], 10.0);
        assert_approx_eq!(vec[3], -15.0);
    }

    #[test]
    #[should_panic]
    fn test_index_mut_out_of_bounds() {
        let vec = Vec4::new(1.0, -4.0, 12.0, -17.0);

        let _ = vec[4];
    }
}
