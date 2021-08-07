//! Three-dimensional vector alias.

use nalgebra::Vector3;

/// Three-dimensional real-number vector.
pub struct Vec3 {
    /// Internal data.
    data: Vector3<f64>,
}

impl Vec3 {
    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            data: Vector3::new(x, y, z),
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
}

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
