//! Time units.

use crate::clone;
use crate::core::Real;
use crate::phys::Length;
use crate::phys::Speed;
use std::ops::Mul;

/// Time primitive unit.
pub struct Time {
    /// Scalar component.
    x: Real,
}

// impl Real {
//     /// Construct a new unit with units of Time.
//     pub fn s(self) -> Time {
//         Time { x: self }
//     }
// }

impl Time {
    clone!(x: Real);

    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub const fn new(x: Real) -> Self {
        Self { x }
    }
}

impl Mul<Speed> for Time {
    type Output = Length;

    #[inline]
    #[must_use]
    fn mul(self, rhs: Speed) -> Self::Output {
        Self::Output::new(self.x * rhs.x())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    pub fn test_new() {
        //let t = 1.0.s();
        //assert_approx_eq!(t.x(), 1.0)
    }
}
