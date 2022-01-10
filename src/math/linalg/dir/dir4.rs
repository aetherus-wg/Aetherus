//! Four-dimensional unit vector.

use crate::core::Real;
use nalgebra::{Unit, Vector4};
use serde_derive::{Deserialize, Serialize};

/// Normalised four dimensional real-number vector.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Dir4 {
    /// Internal data.
    data: Unit<Vector4<Real>>,
}

impl Dir4 {
    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(x: Real, y: Real, z: Real, w: Real) -> Self {
        Self {
            data: Unit::new_normalize(Vector4::new(x, y, z, w)),
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

    /// Access the third component.
    #[inline]
    #[must_use]
    pub fn z(&self) -> Real {
        self.data.z
    }

    /// Access the fourth component.
    #[inline]
    #[must_use]
    pub fn w(&self) -> Real {
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
