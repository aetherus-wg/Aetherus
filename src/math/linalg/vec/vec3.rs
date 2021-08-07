//! Three-dimensional vector alias.

use nalgebra::Vector3;

/// Three-dimensional real-number vector alias.
pub type Vec3 = Vector3<f64>;

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;
    use std::f64::consts::{E, PI, SQRT_2};

    #[test]
    fn test_init() {
        let v = Vec3::new(SQRT_2, PI, E);

        assert_approx_eq!(v.x, SQRT_2);
        assert_approx_eq!(v.y, PI);
        assert_approx_eq!(v.z, E);
    }
}
