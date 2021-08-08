//! Three-dimensional similarity transformation.

use nalgebra::Similarity3;

/// Three-dimensional similarity transformation.
pub struct Trans3 {
    // Internal data.
    data: Similarity3<f64>
}
