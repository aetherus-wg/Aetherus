//! Three-dimensional rotation transformation.

use nalgebra::Rotation3;

/// Three-dimensional rotation transformation.
pub struct Rot3 {
    // Internal data.
    data: Rotation3<f64>
}
