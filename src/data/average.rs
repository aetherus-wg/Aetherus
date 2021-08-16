//! Rolling average implementation.

use crate::{
    clone,
    core::{Int, Real},
};
use std::ops::AddAssign;

///This struct takes a number of samples, of type f64, of some distribution of
/// values and calculates the rolling average of those values.

#[derive(Clone)]
pub struct Average {
    /// The total number of accumulated samples.
    counts: Int,
    /// The total value of all accumulated samples.
    total: Real,
}

impl Average {
    clone!(counts: Int);
    clone!(total: Real);

    /// This constructs a new instance of the Average struct, setting all fields
    /// to zero.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            counts: 0,
            total: 0.0,
        }
    }

    /// Returns the mean value of all accumulated samples.
    #[inline]
    #[must_use]
    pub fn ave(&self) -> Real {
        if self.counts > 0 {
            self.total / Real::from(self.counts)
        } else {
            0.0
        }
    }
}

impl AddAssign for Average {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.total += rhs.total;
        self.counts += rhs.counts;
    }
}

impl AddAssign<&Self> for Average {
    #[inline]
    fn add_assign(&mut self, rhs: &Self) {
        self.total += rhs.total;
        self.counts += rhs.counts;
    }
}

impl AddAssign<Real> for Average {
    #[inline]
    fn add_assign(&mut self, rhs: Real) {
        self.total += rhs;
        self.counts += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    /// This test checks that the constructor works as intended, and that
    /// the fields in the struct are zero-initialised.
    #[test]
    fn test_init() {
        let a = Average::new();

        assert_eq!(a.counts, 0);
        assert_approx_eq!(a.total, 0.0);
    }

    /// This text checks to see that we sensibly handle the edge case where there
    /// are zero accumulated samples, else there may be a divide-by-zero error.
    #[test]
    fn test_zero() {
        let a = Average::new();
        assert_eq!(a.ave(), 0.0);
    }

    /// This test checks to see
    #[test]
    fn test_sum() {
        let mut a = Average::new();

        for n in 0..100 {
            a += Real::from(n);
        }

        assert_eq!(a.counts, 100);
        assert_approx_eq!(a.total, 4950.0);
        assert_approx_eq!(a.ave(), 49.5);
    }
}
