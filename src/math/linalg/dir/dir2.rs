//! Two-dimensional unit vector.

use crate::core::Real;
use nalgebra::{Unit, Vector2};

/// Normalised two dimensional real-number vector.
pub struct Dir2 {
    /// Internal data.
    data: Unit<Vector2<Real>>,
}

impl Dir2 {
    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(x: Real, y: Real) -> Self {
        Self {
            data: Unit::new_normalize(Vector2::new(x, y)),
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
}

impl From<Unit<Vector2<Real>>> for Dir2 {
    #[inline]
    #[must_use]
    fn from(d: Unit<Vector2<Real>>) -> Self {
        Self { data: d }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_init() {
        let v = Dir2::new(3.0, -4.0);

        assert_approx_eq!(v.x(), 0.6);
        assert_approx_eq!(v.y(), -0.8);
    }
}
