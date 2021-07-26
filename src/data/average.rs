//! Rolling average implementation.

use std::ops::AddAssign;

/// Rolling average value recording.
#[derive(Clone, Default)]
pub struct Average {
    /// Current average value.
    total: f64,
    /// Total counts so far.
    counts: i32,
}

impl Average {
    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            total: 0.0,
            counts: 0,
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
