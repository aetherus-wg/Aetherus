//! Square fourth-order matrix.

use nalgebra::{Matrix4};

/// Four-by-four real-number matrix.
pub struct Mat4 {
    /// Internal data.
    data: Matrix4<f64>,
}
