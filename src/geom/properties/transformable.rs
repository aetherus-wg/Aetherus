//! Transformable trait.

use crate::math::Trans3;

/// # Transformable
/// 
/// A trait that indicates that a type may be transformed within the simulation.
/// 
/// Any type implementing this trait can be transformed using the 3-dimensional
/// transform in the `math` module (`crate::math::Trans3`).
/// This is currently performed using an `nalgebra::geometry::Similarity3`, which is a
/// uniform scaling, followed by a rotation, followed by a translation. 
pub trait Transformable {
    /// Apply the given transformation.
    fn transform(&mut self, trans: &Trans3);
}