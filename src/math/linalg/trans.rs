//! Similarity transformation aliases.

use nalgebra::{Similarity2, Similarity3};

/// Two-dimensional transformation alias.
pub type Trans2 = Similarity2<f64>;
/// Three-dimensional transformation alias.
pub type Trans3 = Similarity3<f64>;
