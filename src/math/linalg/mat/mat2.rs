//! Square second-order matrix.

use nalgebra::{Matrix2};

/// Two-by-two real-number matrix.
pub struct Mat2 {
    /// Internal data.
    data: Matrix2<f64>,
}
