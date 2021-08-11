//! Three-dimensional unit vector alias.

use crate::core::Real;
use nalgebra::{Unit, Vector3};

/// Normalised three dimensional real-number vector.
pub struct Dir3 {
    /// Internal data.
    data: Unit<Vector3<Real>>,
}

impl Dir3 {
    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(x: Real, y: Real, z: Real) -> Self {
        Self {
            data: Unit::new_normalize(Vector3::new(x, y, z)),
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
}

impl From<Unit<Vector3<Real>>> for Dir3 {
    #[inline]
    #[must_use]
    fn from(d: Unit<Vector3<Real>>) -> Self {
        Self { data: d }
    }
}
