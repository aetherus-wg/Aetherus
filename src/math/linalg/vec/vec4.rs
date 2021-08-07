//! Four-dimensional vector alias.

use nalgebra::Vector4;


/// Four-dimensional real-number vector.
pub struct Vec4 {
    /// Internal data.
    data: Vector4<f64>,
}

impl Vec4 {
    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self {
            data: Vector4::new(x, y, z, w),
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

    /// Access the third component.
    #[inline]
    #[must_use]
    pub fn z(&self) -> f64 {
        return self.data.z;
    }

    /// Access the fourth component.
    #[inline]
    #[must_use]
    pub fn w(&self) -> f64 {
        return self.data.w;
    }
}

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
