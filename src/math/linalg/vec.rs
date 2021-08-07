//! Vector alias.

use nalgebra::{Vector2, Vector3, Vector4};

/// Two-dimensional real-number vector alias.
pub type Vec2 = Vector2<f64>;
/// Three-dimensional real-number vector alias.
pub type Vec3 = Vector3<f64>;
/// Four-dimensional real-number vector alias.
pub type Vec4 = Vector4<f64>;

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;
    use std::f64::consts::{E, LN_2, PI, SQRT_2};

    #[test]
    fn test_vec2_init() {
        let v = Vec2::new(E, PI);

        assert_approx_eq!(v.x, E);
        assert_approx_eq!(v.y, PI);
    }

    #[test]
    fn test_vec3_init() {
        let v = Vec3::new(E, PI, SQRT_2);

        assert_approx_eq!(v.x, E);
        assert_approx_eq!(v.y, PI);
        assert_approx_eq!(v.z, SQRT_2);
    }

    #[test]
    fn test_vec4_init() {
        let v = Vec4::new(E, PI, SQRT_2, LN_2);

        assert_approx_eq!(v.x, E);
        assert_approx_eq!(v.y, PI);
        assert_approx_eq!(v.z, SQRT_2);
        assert_approx_eq!(v.w, LN_2);
    }

    #[test]
    fn test_vec2_add() {
        let v0 = Vec2::new(0.5, -2.0);
        let v1 = Vec2::new(5.0, 7.0);

        let ans = v0 + v1;

        assert_approx_eq!(ans.x, 5.5);
        assert_approx_eq!(ans.y, 5.0);
    }
}
