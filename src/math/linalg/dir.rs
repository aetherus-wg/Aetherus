//! Unit (normalised) vector alias.

use crate::math::{Vec2, Vec3, Vec4};
use nalgebra::Unit;

/// Normalised two-dimensional vector alias.
pub type Dir2 = Unit<Vec2>;
/// Normalised three-dimensional vector alias.
pub type Dir3 = Unit<Vec3>;
/// Normalised four-dimensional vector alias.
pub type Dir4 = Unit<Vec4>;
