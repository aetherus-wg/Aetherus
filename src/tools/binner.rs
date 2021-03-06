//! Binner implementation.

use crate::{access, clone, tools::Range};
use arctk_attr::file;
use std::fmt::{Display, Error, Formatter};

/// One-dimensional binning structure.
#[file]
#[derive(Clone, PartialEq)]
pub struct Binner {
    /// Range.
    range: Range,
    /// Total number of bins.
    bins: usize,
}

impl Binner {
    access!(range: Range);
    clone!(bins: usize);

    /// Construct a new Binner.
    #[inline]
    #[must_use]
    pub fn new(range: Range, bins: usize) -> Self {
        debug_assert!(bins > 0);

        Self { range, bins }
    }

    /// Calculate the bin width.
    #[inline]
    #[must_use]
    pub fn bin_width(&self) -> f64 {
        self.range.width() / self.bins as f64
    }

    /// Determine the corresponding bin.
    #[inline]
    #[must_use]
    pub fn bin(&self, x: f64) -> usize {
        debug_assert!(self.range.contains(x));

        let frac = (x - self.range.min()) / self.range.width();
        let bin = (frac * self.bins as f64).floor() as usize;
        bin.min(self.bins - 1) as usize
    }

    /// Determine the corresponding bin if the value is within the range.
    #[inline]
    #[must_use]
    pub fn try_bin(&self, x: f64) -> Option<usize> {
        self.range.contains(x).then(|| self.bin(x))
    }
}

impl Display for Binner {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "[{}] ({})", self.range, self.bins)
    }
}
