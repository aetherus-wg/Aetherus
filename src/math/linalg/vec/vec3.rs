//! Three-dimensional vector.

use crate::{clone, core::Real, math::Dir3};
use nalgebra::{Unit, Vector3};
use std::ops::{
    Add, AddAssign, BitXor, BitXorAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg,
    Sub, SubAssign,
};

/// Three-dimensional real-number vector.
pub struct Vec3 {
    /// Internal data.
    data: Vector3<Real>,
}

impl Vec3 {
    clone!(data: Vector3<Real>);

    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub const fn new(x: Real, y: Real, z: Real) -> Self {
        Self {
            data: Vector3::new(x, y, z),
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

    /// Calculate the magnitude of the vector.
    #[inline]
    #[must_use]
    pub fn mag(&self) -> Real {
        self.data.magnitude()
    }

    /// Calculate the unit vector.
    #[inline]
    #[must_use]
    pub fn dir(&self) -> Dir3 {
        Dir3::from(Unit::new_normalize(self.data))
    }
}

impl From<Vector3<Real>> for Vec3 {
    #[inline]
    #[must_use]
    fn from(v: Vector3<Real>) -> Self {
        Self { data: v }
    }
}

impl Neg for Vec3 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn neg(self) -> Self {
        Self::from(-self.data)
    }
}

impl Add for Vec3 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn add(self, rhs: Self) -> Self {
        Self::from(self.data + rhs.data)
    }
}

impl Sub for Vec3 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn sub(self, rhs: Self) -> Self {
        Self::from(self.data - rhs.data)
    }
}

impl Mul<Real> for Vec3 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn mul(self, rhs: Real) -> Self {
        Self::from(self.data * rhs)
    }
}

impl Div<Real> for Vec3 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn div(self, rhs: Real) -> Self {
        Self::from(self.data / rhs)
    }
}

impl AddAssign for Vec3 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.data += rhs.data;
    }
}

impl SubAssign for Vec3 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.data -= rhs.data;
    }
}

impl MulAssign<Real> for Vec3 {
    #[inline]
    fn mul_assign(&mut self, rhs: Real) {
        self.data *= rhs;
    }
}

impl DivAssign<Real> for Vec3 {
    #[inline]
    fn div_assign(&mut self, rhs: Real) {
        self.data /= rhs;
    }
}

impl Mul for Vec3 {
    type Output = Real;

    #[inline]
    #[must_use]
    fn mul(self, rhs: Self) -> Self::Output {
        self.data.dot(&rhs.data)
    }
}

impl BitXor for Vec3 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn bitxor(self, rhs: Self) -> Self {
        Self::new(
            (self.data.y * rhs.data.z) - (self.data.z * rhs.data.y),
            (self.data.z * rhs.data.x) - (self.data.x * rhs.data.z),
            (self.data.x * rhs.data.y) - (self.data.y * rhs.data.x),
        )
    }
}

impl BitXorAssign for Vec3 {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        let x = (self.data.y * rhs.data.z) - (self.data.z * rhs.data.y);
        let y = (self.data.z * rhs.data.x) - (self.data.x * rhs.data.z);
        let z = (self.data.x * rhs.data.y) - (self.data.y * rhs.data.x);

        self.data.x = x;
        self.data.y = y;
        self.data.z = z;
    }
}

impl Index<usize> for Vec3 {
    type Output = Real;

    #[inline]
    #[must_use]
    fn index(&self, i: usize) -> &Self::Output {
        match i {
            0 => &self.data.x,
            1 => &self.data.y,
            2 => &self.data.z,
            _ => panic!("Out of bounds index for three-dimensional vector."),
        }
    }
}

impl IndexMut<usize> for Vec3 {
    #[inline]
    #[must_use]
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        match i {
            0 => &mut self.data.x,
            1 => &mut self.data.y,
            2 => &mut self.data.z,
            _ => panic!("Out of bounds index for three-dimensional vector."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_new() {
        let vec = Vec3::new(0.5, -2.0, 6.0);

        assert_approx_eq!(vec.x(), 0.5);
        assert_approx_eq!(vec.y(), -2.0);
        assert_approx_eq!(vec.z(), 6.0);
    }

    #[test]
    fn test_mag() {
        let vec = Vec3::new(3.0, -4.0, 12.0);

        assert_approx_eq!(vec.mag(), 13.0);
    }

    #[test]
    fn test_dir() {
        let vec = Vec3::new(3.0, -4.0, 12.0);

        let dir = vec.dir();

        assert_approx_eq!(dir.x(), 3.0 / 13.0);
        assert_approx_eq!(dir.y(), -4.0 / 13.0);
        assert_approx_eq!(dir.z(), 12.0 / 13.0);
    }

    #[test]
    fn test_convert() {
        let vec = Vec3::from(Vector3::new(1.23, -4.56, 7.89));

        assert_approx_eq!(vec.x(), 1.23);
        assert_approx_eq!(vec.y(), -4.56);
        assert_approx_eq!(vec.z(), 7.89);
    }

    #[test]
    fn test_neg() {
        let vec = Vec3::new(1.0, -4.0, 12.0);

        let ans = -vec;

        assert_approx_eq!(ans.x(), -1.0);
        assert_approx_eq!(ans.y(), 4.0);
        assert_approx_eq!(ans.z(), -12.0);
    }

    #[test]
    fn test_add() {
        let vec_a = Vec3::new(1.0, -4.0, 12.0);
        let vec_b = Vec3::new(5.0, -7.0, -11.0);

        let ans = vec_a + vec_b;

        assert_approx_eq!(ans.x(), 6.0);
        assert_approx_eq!(ans.y(), -11.0);
        assert_approx_eq!(ans.z(), 1.0);
    }

    #[test]
    fn test_sub() {
        let vec_a = Vec3::new(1.0, -4.0, 12.0);
        let vec_b = Vec3::new(5.0, -7.0, -11.0);

        let ans = vec_a - vec_b;

        assert_approx_eq!(ans.x(), -4.0);
        assert_approx_eq!(ans.y(), 3.0);
        assert_approx_eq!(ans.z(), 23.0);
    }

    #[test]
    fn test_mul() {
        let vec = Vec3::new(1.0, -4.0, 12.0);

        let ans = vec * -5.0;

        assert_approx_eq!(ans.x(), -5.0);
        assert_approx_eq!(ans.y(), 20.0);
        assert_approx_eq!(ans.z(), -60.0);
    }

    #[test]
    fn test_div() {
        let vec = Vec3::new(0.5, -2.0, 2.5);

        let ans = vec / -5.0;

        assert_approx_eq!(ans.x(), -0.1);
        assert_approx_eq!(ans.y(), 0.4);
        assert_approx_eq!(ans.z(), -0.5);
    }

    #[test]
    fn test_add_assign() {
        let mut vec_a = Vec3::new(1.0, -4.0, 12.0);
        let vec_b = Vec3::new(5.0, -7.0, -11.0);

        vec_a += vec_b;

        assert_approx_eq!(vec_a.x(), 6.0);
        assert_approx_eq!(vec_a.y(), -11.0);
        assert_approx_eq!(vec_a.z(), 1.0);
    }

    #[test]
    fn test_sub_assign() {
        let mut vec_a = Vec3::new(1.0, -4.0, 12.0);
        let vec_b = Vec3::new(5.0, -7.0, -11.0);

        vec_a -= vec_b;

        assert_approx_eq!(vec_a.x(), -4.0);
        assert_approx_eq!(vec_a.y(), 3.0);
        assert_approx_eq!(vec_a.z(), 23.0);
    }

    #[test]
    fn test_mul_assign() {
        let mut vec = Vec3::new(1.0, -4.0, 12.0);

        vec *= -5.0;

        assert_approx_eq!(vec.x(), -5.0);
        assert_approx_eq!(vec.y(), 20.0);
        assert_approx_eq!(vec.z(), -60.0);
    }

    #[test]
    fn test_div_assign() {
        let mut vec = Vec3::new(1.0, -4.0, 12.0);

        vec /= -5.0;

        assert_approx_eq!(vec.x(), -0.2);
        assert_approx_eq!(vec.y(), 0.8);
        assert_approx_eq!(vec.z(), -2.4);
    }

    #[test]
    fn test_dot_prod() {
        let vec_a = Vec3::new(1.0, -4.0, 12.0);
        let vec_b = Vec3::new(5.0, -7.0, -11.0);

        let ans = vec_a * vec_b;

        assert_approx_eq!(ans, 5.0 + 28.0 + -132.0);
    }

    #[test]
    fn test_cross_prod() {
        let vec_a = Vec3::new(1.0, -4.0, 12.0);
        let vec_b = Vec3::new(5.0, -7.0, -11.0);

        let ans = vec_a ^ vec_b;

        assert_approx_eq!(ans.x(), 44.0 - -84.0);
        assert_approx_eq!(ans.y(), 60.0 - -11.0);
        assert_approx_eq!(ans.z(), -7.0 - -20.0);
    }

    #[test]
    fn test_cross_prod_assign() {
        let mut vec_a = Vec3::new(1.0, -4.0, 12.0);
        let vec_b = Vec3::new(5.0, -7.0, -11.0);

        vec_a ^= vec_b;

        assert_approx_eq!(vec_a.x(), 44.0 - -84.0);
        assert_approx_eq!(vec_a.y(), 60.0 - -11.0);
        assert_approx_eq!(vec_a.z(), -7.0 - -20.0);
    }

    #[test]
    fn test_index() {
        let vec = Vec3::new(1.0, -4.0, 12.0);

        assert_approx_eq!(vec[0], 1.0);
        assert_approx_eq!(vec[1], -4.0);
        assert_approx_eq!(vec[2], 12.0);
    }

    #[test]
    fn test_index_mut() {
        let mut vec = Vec3::new(1.0, -4.0, 12.0);

        vec[0] *= 2.0;
        vec[1] /= -2.0;
        vec[2] -= 2.0;

        assert_approx_eq!(vec[0], 2.0);
        assert_approx_eq!(vec[1], 2.0);
        assert_approx_eq!(vec[2], 10.0);
    }

    #[test]
    #[should_panic]
    fn test_index_mut_out_of_bounds() {
        let vec = Vec3::new(1.0, -4.0, 12.0);

        let _ = vec[3];
    }
}
