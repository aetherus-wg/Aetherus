//! Collide trait.

use crate::geom::Cube;

// TODO: Would this be better served as being renamed `Intersect`?
/// # Collide
/// Types implementing this trait can be tested for collision with an axis-aligned bounding box.
///
/// As this trait is geometry dependent, we will test the collide implementation per implementing type.
pub trait Collide {
    /// Check for an overlapping collision.
    fn overlap(&self, aabb: &Cube) -> bool;
}
