//! Two-dimensional vector alias.

use crate::{core::Real, math::Dir2};
use nalgebra::{Unit, Vector2};
use std::ops::{
    Add, AddAssign, BitXor, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

/// Two-dimensional real-number vector.
pub struct Vec2 {
    /// Internal data.
    data: Vector2<Real>,
}

impl Vec2 {
    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub const fn new(x: Real, y: Real) -> Self {
        Self {
            data: Vector2::new(x, y),
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

    /// Calculate the magnitude of the vector.
    #[inline]
    #[must_use]
    pub fn mag(&self) -> Real {
        self.data.magnitude()
    }

    /// Calculate the unit vector.
    #[inline]
    #[must_use]
    pub fn dir(&self) -> Dir2 {
        Dir2::from(Unit::new_normalize(self.data))
    }
}

impl From<Vector2<Real>> for Vec2 {
    #[inline]
    #[must_use]
    fn from(v: Vector2<Real>) -> Self {
        Self { data: v }
    }
}

impl Neg for Vec2 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn neg(self) -> Self {
        Self::from(-self.data)
    }
}

impl Add for Vec2 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn add(self, rhs: Self) -> Self {
        Self::from(self.data + rhs.data)
    }
}

impl Sub for Vec2 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn sub(self, rhs: Self) -> Self {
        Self::from(self.data - rhs.data)
    }
}

impl Mul<Real> for Vec2 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn mul(self, rhs: Real) -> Self {
        Self::from(self.data * rhs)
    }
}

impl Div<Real> for Vec2 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn div(self, rhs: Real) -> Self {
        Self::from(self.data / rhs)
    }
}

impl AddAssign<Self> for Vec2 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.data += rhs.data;
    }
}

impl SubAssign<Self> for Vec2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.data -= rhs.data;
    }
}

impl MulAssign<Real> for Vec2 {
    #[inline]
    fn mul_assign(&mut self, rhs: Real) {
        self.data *= rhs;
    }
}

impl DivAssign<Real> for Vec2 {
    #[inline]
    fn div_assign(&mut self, rhs: Real) {
        self.data /= rhs;
    }
}

impl Mul for Vec2 {
    type Output = Real;

    #[inline]
    #[must_use]
    fn mul(self, rhs: Self) -> Self::Output {
        self.data.dot(&rhs.data)
    }
}

impl BitXor for Vec2 {
    type Output = Real;

    #[inline]
    #[must_use]
    fn bitxor(self, rhs: Self) -> Self::Output {
        (self.data.x * rhs.data.y) - (self.data.y * rhs.data.x)
    }
}

impl Index<usize> for Vec2 {
    type Output = Real;

    #[inline]
    #[must_use]
    fn index(&self, i: usize) -> &Self::Output {
        match i {
            0 => &self.data.x,
            1 => &self.data.y,
            _ => panic!("Out of bounds index for two-dimensional vector."),
        }
    }
}

impl IndexMut<usize> for Vec2 {
    #[inline]
    #[must_use]
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        match i {
            0 => &mut self.data.x,
            1 => &mut self.data.y,
            _ => panic!("Out of bounds index for two-dimensional vector."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_new() {
        let vec = Vec2::new(0.5, -2.0);

        assert_approx_eq!(vec.x(), 0.5);
        assert_approx_eq!(vec.y(), -2.0);
    }

    #[test]
    fn test_mag() {
        let vec = Vec2::new(3.0, -4.0);

        assert_approx_eq!(vec.mag(), 5.0);
    }

    #[test]
    fn test_dir() {
        let vec = Vec2::new(3.0, -4.0);

        let dir = vec.dir();

        assert_approx_eq!(dir.x(), 0.6);
        assert_approx_eq!(dir.y(), -0.8);
    }

    #[test]
    fn test_convert() {
        let vec = Vec2::from(Vector2::new(1.23, -4.56));

        assert_approx_eq!(vec.x(), 1.23);
        assert_approx_eq!(vec.y(), -4.56);
    }

    #[test]
    fn test_neg() {
        let vec = Vec2::new(1.0, -4.0);

        let ans = -vec;

        assert_approx_eq!(ans.x(), -1.0);
        assert_approx_eq!(ans.y(), 4.0);
    }

    #[test]
    fn test_add() {
        let vec_a = Vec2::new(1.0, -4.0);
        let vec_b = Vec2::new(5.0, -7.0);

        let ans = vec_a + vec_b;

        assert_approx_eq!(ans.x(), 6.0);
        assert_approx_eq!(ans.y(), -11.0);
    }

    #[test]
    fn test_sub() {
        let vec_a = Vec2::new(1.0, -4.0);
        let vec_b = Vec2::new(5.0, -7.0);

        let ans = vec_a - vec_b;

        assert_approx_eq!(ans.x(), -4.0);
        assert_approx_eq!(ans.y(), 3.0);
    }

    #[test]
    fn test_mul() {
        let vec = Vec2::new(1.0, -4.0);

        let ans = vec * -5.0;

        assert_approx_eq!(ans.x(), -5.0);
        assert_approx_eq!(ans.y(), 20.0);
    }

    #[test]
    fn test_div() {
        let vec = Vec2::new(0.5, -2.0);

        let ans = vec / -5.0;

        assert_approx_eq!(ans.x(), -0.1);
        assert_approx_eq!(ans.y(), 0.4);
    }

    #[test]
    fn test_add_assign() {
        let mut vec_a = Vec2::new(1.0, -4.0);
        let vec_b = Vec2::new(5.0, -7.0);

        vec_a += vec_b;

        assert_approx_eq!(vec_a.x(), 6.0);
        assert_approx_eq!(vec_a.y(), -11.0);
    }

    #[test]
    fn test_sub_assign() {
        let mut vec_a = Vec2::new(1.0, -4.0);
        let vec_b = Vec2::new(5.0, -7.0);

        vec_a -= vec_b;

        assert_approx_eq!(vec_a.x(), -4.0);
        assert_approx_eq!(vec_a.y(), 3.0);
    }

    #[test]
    fn test_mul_assign() {
        let mut vec = Vec2::new(1.0, -4.0);

        vec *= -5.0;

        assert_approx_eq!(vec.x(), -5.0);
        assert_approx_eq!(vec.y(), 20.0);
    }

    #[test]
    fn test_div_assign() {
        let mut vec = Vec2::new(1.0, -4.0);

        vec /= -5.0;

        assert_approx_eq!(vec.x(), -0.2);
        assert_approx_eq!(vec.y(), 0.8);
    }

    #[test]
    fn test_dot_prod() {
        let vec_a = Vec2::new(1.0, -4.0);
        let vec_b = Vec2::new(5.0, -7.0);

        let ans = vec_a * vec_b;

        assert_approx_eq!(ans, 5.0 + 28.0);
    }

    #[test]
    fn test_cross_prod() {
        let vec_a = Vec2::new(1.0, -4.0);
        let vec_b = Vec2::new(5.0, -7.0);

        let ans = vec_a ^ vec_b;

        assert_approx_eq!(ans, -7.0 - -20.0);
    }

    #[test]
    fn test_index() {
        let vec = Vec2::new(1.0, -4.0);

        assert_approx_eq!(vec[0], 1.0);
        assert_approx_eq!(vec[1], -4.0);
    }

    #[test]
    fn test_index_mut() {
        let mut vec = Vec2::new(1.0, -4.0);

        vec[0] *= 2.0;
        vec[1] /= -2.0;

        assert_approx_eq!(vec[0], 2.0);
        assert_approx_eq!(vec[1], 2.0);
    }

    #[test]
    #[should_panic]
    fn test_index_mut_out_of_bounds() {
        let vec = Vec2::new(1.0, -4.0);

        let _ = vec[2];
    }
}
