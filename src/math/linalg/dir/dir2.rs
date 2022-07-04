//! Two-dimensional unit vector.

use crate::{clone, core::Real};
use nalgebra::{Unit, Vector2};
use serde_derive::{Deserialize, Serialize};
use std::{
    ops::{Neg, Sub}
};

/// Normalised two dimensional real-number vector.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Dir2 {
    /// Internal data.
    data: Unit<Vector2<Real>>,
}

impl Dir2 {
    clone!(data: Unit<Vector2<Real>>);

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

    #[inline]
    #[must_use]
    pub fn dot(&self, b: &Dir2) -> f64 {
        self.data.dot(&b.data)
    }
}

impl From<Unit<Vector2<Real>>> for Dir2 {
    #[inline]
    #[must_use]
    fn from(d: Unit<Vector2<Real>>) -> Self {
        Self { data: d }
    }
}

impl Neg for Dir2 {
    type Output = Self;

    /// Negation implementation for Dir2. 
    #[inline]
    #[must_use]
    fn neg(self) -> Self::Output {
        return Self::new(-self.x(), -self.y());
    }
}

impl Sub<Dir2> for Dir2 {
    type Output = Self;

    #[inline]
    #[must_use]
    fn sub(self, rhs: Dir2) -> Self::Output {
        Self::new(self.x() - rhs.x(), self.y() - rhs.y())
    }
}

impl PartialEq for Dir2 {
    #[inline]
    #[must_use]
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
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

    #[test]
    fn test_dir2_neg() {
        let test_pos = Dir2::new(1.0, 1.0);
        let test_neg = Dir2::new(-1.0, -1.0);

        // First test that positive components get made negative.
        assert_approx_eq!(-test_pos.x(), test_neg.x());
        assert_approx_eq!(-test_pos.y(), test_neg.y());

        // Now test the inverse. 
        assert_approx_eq!(-test_neg.x(), test_pos.x());
        assert_approx_eq!(-test_neg.y(), test_pos.y());
    }
}
