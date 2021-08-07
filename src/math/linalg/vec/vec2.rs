//! Two-dimensional vector alias.

use nalgebra::Vector2;
use std::ops::Add;

/// Two-dimensional real-number vector.
pub struct Vec2 {
    /// Internal data.
    data: Vector2<f64>
}

impl Vec2 {
    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(x: f64, y:f64) -> Self {
        Self{
            data: Vector2::new(x, y)
        }
    }

    /// Access the first component.
    #[inline]
    #[must_use]
    pub fn x(&self) -> f64 {
        return self.data.x
    }

    /// Access the second component.
    #[inline]
    #[must_use]
    pub fn y(&self) -> f64 {
        return self.data.y
    }
}

impl From<Vector2<f64>> for Vec2 {
    fn from(v: Vector2<f64>) -> Self {
        Self{
            data: v
        }
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        return Self::from(self.data + rhs.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;
    use std::f64::consts::{PI, SQRT_2};

    #[test]
    fn test_init() {
        let v = Vec2::new(SQRT_2, PI);

        assert_approx_eq!(v.x(), SQRT_2);
        assert_approx_eq!(v.y(), PI);
    }

    #[test]
    fn test_convert() {
        let v = Vec2::from(Vector2::new(1.23, -4.56));

        assert_approx_eq!(v.x(), 1.23);
        assert_approx_eq!(v.y(), -4.56);
    }

    #[test]
    fn test_add() {
        let v0 = Vec2::new(0.5, -2.0);
        let v1 = Vec2::new(5.0, 7.0);

        let ans = v0 + v1;

        assert_approx_eq!(ans.x(), 5.5);
        assert_approx_eq!(ans.y(), 5.0);
    }
}
