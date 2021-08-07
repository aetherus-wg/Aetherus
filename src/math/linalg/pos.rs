//! Point position alias.

use nalgebra::{Point2, Point3, Point4};

/// Two-dimensional real-number position alias.
pub type Pos2 = Point2<f64>;
/// Three-dimensional real-number position alias.
pub type Pos3 = Point3<f64>;
/// Four-dimensional real-number position alias.
pub type Pos4 = Point4<f64>;

/// Two-dimensional discrete-number position alias.
pub type Pos2I = Point2<i32>;
/// Three-dimensional discrete-number position alias.
pub type Pos3I = Point3<i32>;
/// Four-dimensional discrete-number position alias.
pub type Pos4I = Point4<i32>;
