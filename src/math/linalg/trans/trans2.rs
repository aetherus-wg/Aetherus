//! Two-dimensional similarity transformation.

use nalgebra::Similarity2;

/// Two-dimensional similarity transformation.
pub struct Trans2 {
    // Internal data.
    data: Similarity2<f64>
}
