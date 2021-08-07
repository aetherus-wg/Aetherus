//! Two-dimensional vector alias.

use crate::math::Dir2;
use nalgebra::{Unit, Vector2};
use std::ops::{
    Add, AddAssign, BitXor, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

/// Two-dimensional real-number vector.
pub struct Vec2 {
    /// Internal data.
    data: Vector2<f64>,
}

impl Vec2 {
    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            data: Vector2::new(x, y),
        }
    }

    /// Access the first component.
    #[inline]
    #[must_use]
    pub fn x(&self) -> f64 {
        return self.data.x;
    }

    /// Access the second component.
    #[inline]
    #[must_use]
    pub fn y(&self) -> f64 {
        return self.data.y;
    }

    /// Calculate the magnitude of the vector.
    #[inline]
    #[must_use]
    pub fn mag(&self) -> f64 {
        self.data.magnitude()
    }

    /// Calculate the unit vector.
    #[inline]
    #[must_use]
    pub fn dir(&self) -> Dir2 {
        Dir2::from(Unit::new_normalize(self.data))
    }
}

impl From<Vector2<f64>> for Vec2 {
    fn from(v: Vector2<f64>) -> Self {
        Self { data: v }
    }
}

impl Neg for Vec2 {
    type Output = Self;

    fn neg(self) -> Self {
        return Self::from(-self.data);
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        return Self::from(self.data + rhs.data);
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        return Self::from(self.data - rhs.data);
    }
}

impl Mul<f64> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        return Self::from(self.data * rhs);
    }
}

impl Div<f64> for Vec2 {
    type Output = Self;

    fn div(self, rhs: f64) -> Self {
        return Self::from(self.data / rhs);
    }
}

impl AddAssign<Self> for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.data += rhs.data;
    }
}

impl SubAssign<Self> for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.data -= rhs.data;
    }
}

impl MulAssign<f64> for Vec2 {
    fn mul_assign(&mut self, rhs: f64) {
        self.data *= rhs;
    }
}

impl DivAssign<f64> for Vec2 {
    fn div_assign(&mut self, rhs: f64) {
        self.data /= rhs
    }
}

impl Mul for Vec2 {
    type Output = f64;

    fn mul(self, rhs: Self) -> Self::Output {
        return self.data.dot(&rhs.data);
    }
}

impl BitXor for Vec2 {
    type Output = f64;

    fn bitxor(self, rhs: Self) -> Self::Output {
        return (self.data.x * rhs.data.y) - (rhs.data.x * self.data.y);
    }
}

impl Index<usize> for Vec2 {
    type Output = f64;

    fn index(&self, i: usize) -> &Self::Output {
        match i {
            0 => &self.data.x,
            1 => &self.data.y,
            _ => panic!("Out of bounds index for two-dimensional vector."),
        }
    }
}

impl IndexMut<usize> for Vec2 {
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
    use std::f64::consts::{PI, SQRT_2};

    #[test]
    fn test_init() {
        let vec = Vec2::new(SQRT_2, PI);

        assert_approx_eq!(vec.x(), SQRT_2);
        assert_approx_eq!(vec.y(), PI);
    }

    #[test]
    #[should_panic]
    fn test_mag() {
        let vec = Vec2::new(1.0, -4.0);

        assert_approx_eq!(vec.mag(), 2.0);
    }

    #[test]
    #[should_panic]
    fn test_dir() {
        let vec = Vec2::new(1.0, -4.0);

        let dir = vec.dir();

        assert_approx_eq!(dir.x(), 0.5);
        assert_approx_eq!(dir.y(), -2.0);
    }

    #[test]
    fn test_convert() {
        let vec = Vec2::from(Vector2::new(1.23, -4.56));

        assert_approx_eq!(vec.x(), 1.23);
        assert_approx_eq!(vec.y(), -4.56);
    }

    #[test]
    fn test_neg() {
        let vec = Vec2::new(0.5, -2.0);

        let ans = -vec;

        assert_approx_eq!(ans.x(), -0.5);
        assert_approx_eq!(ans.y(), 2.0);
    }

    #[test]
    fn test_add() {
        let vec_a = Vec2::new(0.5, -2.0);
        let vec_b = Vec2::new(5.0, 7.0);

        let ans = vec_a + vec_b;

        assert_approx_eq!(ans.x(), 5.5);
        assert_approx_eq!(ans.y(), 5.0);
    }

    #[test]
    fn test_sub() {
        let vec_a = Vec2::new(0.5, -2.0);
        let vec_b = Vec2::new(5.0, 7.0);

        let ans = vec_a - vec_b;

        assert_approx_eq!(ans.x(), -4.5);
        assert_approx_eq!(ans.y(), -9.0);
    }

    #[test]
    fn test_mul() {
        let vec = Vec2::new(0.5, -2.0);

        let ans = vec * 5.0;

        assert_approx_eq!(ans.x(), 2.5);
        assert_approx_eq!(ans.y(), -10.0);
    }

    #[test]
    fn test_div() {
        let vec = Vec2::new(0.5, -2.0);

        let ans = vec / 5.0;

        assert_approx_eq!(ans.x(), 0.1);
        assert_approx_eq!(ans.y(), -0.4);
    }

    #[test]
    fn test_add_assign() {
        let mut vec_a = Vec2::new(0.5, -2.0);
        let vec_b = Vec2::new(5.0, 7.0);

        vec_a += vec_b;

        assert_approx_eq!(vec_a.x(), 5.5);
        assert_approx_eq!(vec_a.y(), 5.0);
    }

    #[test]
    fn test_sub_assign() {
        let mut vec_a = Vec2::new(0.5, -2.0);
        let vec_b = Vec2::new(5.0, 7.0);

        vec_a -= vec_b;

        assert_approx_eq!(vec_a.x(), -4.5);
        assert_approx_eq!(vec_a.y(), -9.0);
    }

    #[test]
    fn test_mul_assign() {
        let mut vec = Vec2::new(0.5, -2.0);

        vec *= 5.0;

        assert_approx_eq!(vec.x(), 2.5);
        assert_approx_eq!(vec.y(), -10.0);
    }

    #[test]
    fn test_div_assign() {
        let mut vec = Vec2::new(0.5, -2.0);

        vec /= 5.0;

        assert_approx_eq!(vec.x(), 0.1);
        assert_approx_eq!(vec.y(), -0.4);
    }

    #[test]
    fn test_cross_product() {
        let mut vec = Vec2::new(0.5, -2.0);

        vec /= 5.0;

        assert_approx_eq!(vec.x(), 0.1);
        assert_approx_eq!(vec.y(), -0.4);
    }

    #[test]
    fn test_dot_prod() {
        let vec_a = Vec2::new(0.5, -2.0);
        let vec_b = Vec2::new(5.0, 7.0);

        let ans = vec_a * vec_b;

        assert_approx_eq!(ans, 2.5 - 14.0);
    }

    #[test]
    fn test_cross_prod() {
        let vec_a = Vec2::new(0.5, -2.0);
        let vec_b = Vec2::new(5.0, 7.0);

        let ans = vec_a ^ vec_b;

        assert_approx_eq!(ans, 3.5 + 10.0);
    }

    #[test]
    fn test_index() {
        let vec = Vec2::new(0.5, -2.0);

        assert_approx_eq!(vec[0], 0.5);
        assert_approx_eq!(vec[1], -2.0);
    }

    #[test]
    fn test_index_mut() {
        let mut vec = Vec2::new(0.5, -2.0);

        vec[0] *= 2.0;
        vec[1] /= 2.0;

        assert_approx_eq!(vec[0], 1.0);
        assert_approx_eq!(vec[1], -1.0);
    }

    #[test]
    #[should_panic]
    fn test_index_mut_out_of_bounds() {
        let vec = Vec2::new(0.5, -2.0);

        vec[2];
    }
}