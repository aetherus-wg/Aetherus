//! Four-dimensional point position.

use nalgebra::Point4;

/// Four-coordinate position.
pub struct Pos4 {
    // Internal data.
    data: Point4<f64>,
}

impl Pos4 {
    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self {
            data: Point4::new(x, y, z, w),
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

    /// Access the fourth component.
    #[inline]
    #[must_use]
    pub fn w(&self) -> f64 {
        return self.data.w;
    }
}
