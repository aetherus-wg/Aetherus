//! Four-dimensional unit vector alias.

use crate::core::Real;
use nalgebra::{Unit, Vector4};

/// Normalised four dimensional real-number vector.
pub struct Dir4 {
    /// Internal data.
    data: Unit<Vector4<f64>>,
}

impl Dir4 {
    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self {
            data: Unit::new_normalize(Vector4::new(x, y, z, w)),
        }
    }

    /// Access the first component.
    #[inline]
    #[must_use]
    pub fn x(&self) -> f64 {
        self.data.x
    }

    /// Access the second component.
    #[inline]
    #[must_use]
    pub fn y(&self) -> f64 {
        self.data.y
    }

    /// Access the third component.
    #[inline]
    #[must_use]
    pub fn z(&self) -> f64 {
        self.data.z
    }

    /// Access the fourth component.
    #[inline]
    #[must_use]
    pub fn w(&self) -> f64 {
        self.data.w
    }
}

impl From<Unit<Vector4<Real>>> for Dir4 {
    #[inline]
    #[must_use]
    fn from(d: Unit<Vector4<Real>>) -> Self {
        Self { data: d }
    }
}
