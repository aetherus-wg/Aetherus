//! Complex composite type.

use crate::{access, core::Real};
use std::ops::{Mul};

/// Complex number type.
pub struct Complex {
    /// Real component.
    real: Real,
    /// Imaginary component.
    imag: Real,
}

impl Complex {
    access!(real, Real);
    access!(imag, Real);
}

impl Mul for Complex {
    type Output = Self;

    #[inline]
    #[must_use]
    fn mul(self, rhs: Self) -> Self {
        Self {
            real: (self.real * rhs.real) - (self.imag * rhs.imag),
            imag: (self.real * rhs.imag) + (self.imag * rhs.real),
        }
    }
}
