//! Two-dimensional vector alias.

use nalgebra::Vector2;

/// Two-dimensional real-number vector alias.
pub type Vec2 = Vector2<f64>;

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;
    use std::f64::consts::{PI, SQRT_2};

    #[test]
    fn test_init() {
        let v = Vec2::new(SQRT_2, PI);

        assert_approx_eq!(v.x, SQRT_2);
        assert_approx_eq!(v.y, PI);
    }

    #[test]
    fn test_add() {
        let v0 = Vec2::new(0.5, -2.0);
        let v1 = Vec2::new(5.0, 7.0);

        let ans = v0 + v1;

        assert_approx_eq!(ans.x, 5.5);
        assert_approx_eq!(ans.y, 5.0);
    }
}
