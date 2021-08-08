//! Two-dimensional rotation transformation.

use nalgebra::Rotation2;

/// Two-dimensional rotation transformation.
pub struct Rot2 {
    // Internal data.
    data: Rotation2<f64>,
}
