//! Two-dimensional unit vector alias.

use nalgebra::{Unit, Vector2};

/// Normalised two dimensional real-number vector.
pub struct Dir2 {
    /// Internal data.
    data: Unit<Vector2<f64>>,
}

impl Dir2 {
    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            data: Unit::new_normalize(Vector2::new(x, y)),
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
}

impl From<Unit<Vector2<f64>>> for Dir2 {
    fn from(d: Unit<Vector2<f64>>) -> Self {
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
