//! Trace trait

use crate::geom::{Ray, Side};

/// # Trace
///
/// This should be implemented by any type that you want to be able to ray-traced
/// with the hit-scan system. This interface is used when Aetherus wishes to see
/// if a given `Ray` hits an object during a hit scan. Given a hit, the object
/// should then be prepared to provide its distance to the surface along the ray's
/// line of travel, and the side with which it collides.
///
/// As this is implemented differently for each type, we will implement tests for
/// this for each implementing type.
pub trait Trace {
    /// Determine if a ray hit occurs.
    fn hit(&self, ray: &Ray) -> bool;

    /// Distance to the surface along the ray's line of travel.
    fn dist(&self, ray: &Ray) -> Option<f64>;

    /// Distance to the surface along the ray's line of travel and side of collision.
    fn dist_side(&self, ray: &Ray) -> Option<(f64, Side)>;
}
