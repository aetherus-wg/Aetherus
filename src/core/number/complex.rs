//! Complex composite type.

use crate::{access, core::Real};
use std::ops::Mul;

/// Complex number type.
pub struct Complex {
    /// Real component.
    re: Real,
    /// Imaginary component.
    im: Real,
}

impl Complex {
    access!(re: Real);
    access!(im: Real);
}

impl Mul for Complex {
    type Output = Self;

    #[inline]
    #[must_use]
    fn mul(self, rhs: Self) -> Self {
        Self {
            re: (self.re * rhs.re) - (self.im * rhs.im),
            im: (self.re * rhs.im) + (self.im * rhs.re),
        }
    }
}
