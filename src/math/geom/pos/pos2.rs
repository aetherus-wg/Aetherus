//! Two-dimensional position.

use crate::core::{Length, Real};
use nalgebra::{Point2};

/// Two dimensional position.
pub struct Pos2 {
    /// Internal data.
    data: Point2<Real>,
}

impl Pos2 {
    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(x: Length, y: Length) -> Self {
        Self {
            data: Point2::new(x, y),
        }
    }

    /// Access the first component.
    #[inline]
    #[must_use]
    pub fn x(&self) -> Length {
        self.data.x
    }

    /// Access the second component.
    #[inline]
    #[must_use]
    pub fn y(&self) -> Length {
        self.data.y
    }
}

impl From<Point2<Real>> for Pos2 {
    #[inline]
    #[must_use]
    fn from(p: Point2<Real>) -> Self {
        Self { data: p }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use assert_approx_eq::assert_approx_eq;

//     #[test]
//     fn test_init() {
//         let v = Dir2::new(3.0, -4.0);

//         assert_approx_eq!(v.x(), 0.6);
//         assert_approx_eq!(v.y(), -0.8);
//     }
// }
