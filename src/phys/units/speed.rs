//! Speed units.

use crate::clone;
use crate::core::Real;
use crate::phys::Length;
use crate::phys::Time;
use std::ops::Mul;

/// Speed primitive unit.
pub struct Speed {
    /// Scalar component.
    x: Real,
}

impl Speed {
    clone!(x: Real);

    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub const fn new(x: Real) -> Self {
        Self { x }
    }
}

impl Mul<Time> for Speed {
    type Output = Length;

    #[inline]
    #[must_use]
    fn mul(self, rhs: Time) -> Self::Output {
        Self::Output::new(self.x * rhs.x())
    }
}
