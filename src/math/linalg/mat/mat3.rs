//! Square third-order matrix.

use nalgebra::Matrix3;

/// Three-by-three real-number matrix.
pub struct Mat3 {
    /// Internal data.
    data: Matrix3<f64>,
}
