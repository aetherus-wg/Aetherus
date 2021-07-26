//! Rolling average implementation.

use std::ops::AddAssign;

/// Rolling average value recording.
#[derive(Clone, Default)]
pub struct Average {
    /// Total individual contributions so far.
    counts: i32,
    /// Current average value.
    total: f64,
}

impl Average {
    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            counts: 0,
            total: 0.0,
        }
    }

    /// Calculate the average value.
    #[inline]
    #[must_use]
    pub fn ave(&self) -> f64 {
        if self.counts > 0 {
            self.total / f64::from(self.counts)
        } else {
            0.0
        }
    }
}

impl AddAssign<Self> for Average {
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

impl AddAssign<f64> for Average {
    #[inline]
    fn add_assign(&mut self, rhs: f64) {
        self.total += rhs;
        self.counts += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_init() {
        let a = Average::new();

        assert_eq!(a.counts, 0);
        assert_approx_eq!(a.total, 0.0);
    }

    #[test]
    fn test_sum() {
        let mut a = Average::new();

        for n in 0..100 {
            a += f64::from(n);
        }

        assert_eq!(a.counts, 100);
        assert_approx_eq!(a.total, 4950.0);
        assert_approx_eq!(a.ave(), 49.5);
    }
}
