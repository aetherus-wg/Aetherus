//! Length units.

use crate::clone;
use crate::core::Real;
use crate::phys::Speed;
use crate::phys::Time;
use std::ops::Div;

/// Length primitive unit.
pub struct Length {
    /// Scalar component.
    x: Real,
}

impl Length {
    clone!(x: Real);

    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub const fn new(x: Real) -> Self {
        Self { x }
    }
}

impl Div<Time> for Length {
    type Output = Speed;

    #[inline]
    #[must_use]
    fn div(self, rhs: Time) -> Self::Output {
        Self::Output::new(self.x / rhs.x())
    }
}
