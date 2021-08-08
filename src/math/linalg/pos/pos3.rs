//! Three-dimensional point position.

use nalgebra::Point3;

/// Three-coordinate position.
pub struct Pos3 {
    // Internal data.
    data: Point3<f64>
}

impl Pos3 {
    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            data: Point3::new(x, y, z),
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

    /// Access the third component.
    #[inline]
    #[must_use]
    pub fn z(&self) -> f64 {
        return self.data.z;
    }
}
