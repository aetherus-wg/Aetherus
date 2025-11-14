//! Complex composite type.

use crate::{access, core::Real};
use std::ops::Mul;

/// Complex number type.
#[derive(Debug, PartialEq)]
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
    fn mul(self, rhs: Self) -> Self {
        Self {
            re: self.re.mul_add(rhs.re, -self.im * rhs.im),
            im: self.re.mul_add(rhs.im, self.im * rhs.re),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Complex;

    #[test]
    fn get_set_complex() {
        let complex_number = Complex{re: 2.0, im: 3.0};
        assert_eq!(*complex_number.re(), 2.0);
        assert_eq!(*complex_number.im(), 3.0);
    }

    #[test]
    fn test_complex_mul() {
        // Complex multiplication follows the pattern of
        // xy = (a + ib)(c + id)
        //    = (ac - bd) + i(ad + bc)
        let a = 2.0;
        let b = 5.0;
        let c = 3.0;
        let d = 4.0;
        let x = Complex{ re: a, im: b};
        let y = Complex{ re: c, im: d};

        assert_eq!(x * y, Complex{re:(a * c - b * d), im: (a * d + b * c)});
    }

}
