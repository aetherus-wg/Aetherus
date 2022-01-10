//! Hit-scan result enumeration.

use crate::geom::Hit;

/// # Hit-scan result enumeration
///
/// An enum that describes the nature of a hit resulting from a hit-scan.
/// In this case, are we reaching the boundary of a grid cell, or are we hitting
/// the surface of an interface between media.
#[derive(Clone)]
pub enum Scan<'a, T> {
    /// Boundary collision.
    Boundary(f64),
    /// Surface collision.
    Surface(Hit<'a, T>),
}

impl<'a, T> Scan<'a, T> {
    /// Construct a new cell boundary detection instance.
    #[inline]
    #[must_use]
    pub fn new_boundary(dist: f64) -> Self {
        debug_assert!(dist > 0.0);

        Self::Boundary(dist)
    }

    /// Construct a new surface detection instance.
    #[inline]
    #[must_use]
    pub fn new_surface(hit: Hit<'a, T>) -> Self {
        debug_assert!(hit.dist() > 0.0);

        Self::Surface(hit)
    }
}
