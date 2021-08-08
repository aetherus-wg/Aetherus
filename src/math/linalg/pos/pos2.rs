//! Two-dimensional point position.

use nalgebra::Point2;

/// Two-coordinate position.
pub struct Pos2 {
    // Internal data.
    data: Point2<f64>
}

impl Pos2 {
    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            data: Point2::new(x, y),
        }
    }

    /// Access the first component.
    #[inline]
    #[must_use]
    pub fn x(&self) -> f64 {
        return self.data.x;
    }

    /// Access the second component.
    #[inline]
    #[must_use]
    pub fn y(&self) -> f64 {
        return self.data.y;
    }
}
