//! Hit-scan result enumeration.

use crate::geom::Hit;

/// # Hit-scan result enumeration
///
/// An enum that describes the nature of a hit resulting from a hit-scan.
/// In this case, are we reaching the boundary of a grid cell, or are we hitting
/// the surface of an interface between media.
#[derive(Debug, Clone, PartialEq)]
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
        debug_assert!(dist >= 0.0);

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

#[cfg(test)]
mod tests {
    use crate::{sim::Attribute, geom::rt::Side, math::Dir3};
    use super::*;

    #[test]
    fn test_new_boundary() {
        let dist = 1.0;
        let scan: Scan<'_, Attribute> = Scan::new_boundary(dist);
        assert_eq!(scan, Scan::Boundary(dist));
    }

    #[test]
    fn test_new_surface() {
        let hit = Hit::new(&Attribute::Mirror(0.5), 1.0, Side::Inside(Dir3::new(0.0, 0.0, 1.0)));
        let scan = Scan::new_surface(hit.clone());
        assert_eq!(scan, Scan::Surface(hit));
    }

    #[test]
    #[should_panic]
    fn test_new_boundary_zero_dist() {
        let dist = 0.0;
        let _scan: Scan<'_, Attribute> = Scan::new_boundary(dist);
    }

    #[test]
    #[should_panic]
    fn test_new_surface_zero_dist() {
        let hit = Hit::new(&Attribute::Mirror(0.5), 0.0, Side::Inside(Dir3::new(0.0, 0.0, 1.0)));
        let _scan: Scan<'_, Attribute> = Scan::new_surface(hit);
    }
}
