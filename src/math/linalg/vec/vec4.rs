//! Four-dimensional vector alias.

use nalgebra::Vector4;

/// Four-dimensional real-number vector alias.
pub type Vec4 = Vector4<f64>;

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;
    use std::f64::consts::{E, LN_2, PI, SQRT_2};

    #[test]
    fn test_init() {
        let v = Vec4::new(SQRT_2, PI, E, LN_2);

        assert_approx_eq!(v.x, SQRT_2);
        assert_approx_eq!(v.y, PI);
        assert_approx_eq!(v.z, E);
        assert_approx_eq!(v.w, LN_2);
    }
}
