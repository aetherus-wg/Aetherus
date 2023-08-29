//! Range implementation.

use crate::clone;
use arctk_attr::file;
use std::{
    f64::{INFINITY, NEG_INFINITY},
    fmt::{Display, Formatter, Result},
};

/// One-dimensional inclusive Range.
#[file]
#[derive(Clone, PartialEq)]
pub struct Range {
    /// Minimum bound.
    min: f64,
    /// Maximum bound.
    max: f64,
}

impl Range {
    clone!(min: f64);
    clone!(max: f64);

    /// Construct a new Range.
    #[inline]
    #[must_use]
    pub fn new(min: f64, max: f64) -> Self {
        debug_assert!(min < max);

        Self { min, max }
    }

    /// Construct an infinite Range.
    #[inline]
    #[must_use]
    pub const fn new_infinite() -> Self {
        Self {
            min: NEG_INFINITY,
            max: INFINITY,
        }
    }

    /// Calculate the width of the Range.
    #[inline]
    #[must_use]
    pub fn width(&self) -> f64 {
        self.max - self.min
    }

    /// Determine if a value is contained within the Range.
    #[inline]
    #[must_use]
    pub fn contains(&self, x: f64) -> bool {
        if x < self.min || x > self.max {
            return false;
        }

        true
    }

    /// Determine if the Range intersects with another given Range.
    #[inline]
    #[must_use]
    pub fn intersect(&self, other: &Self) -> bool {
        if self.max < other.min || other.max < self.min {
            return false;
        }

        true
    }

    /// From a range of overlapping values.
    #[inline]
    #[must_use]
    pub fn overlap(&self, other: &Self) -> Option<Self> {
        if !self.intersect(other) {
            return None;
        }

        let min = self.min.max(other.min);
        let max = self.max.min(other.max);

        Some(Self::new(min, max))
    }
}

impl Display for Range {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        write!(fmt, "{} -> {}", self.min, self.max)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_new_range() {
        use super::Range;

        let min = 0.0;
        let max = 1.0;

        let range = Range::new(min, max);

        assert_eq!(range.min(), min);
        assert_eq!(range.max(), max);
    }

    #[test]
    #[should_panic]
    fn test_new_range_fail() {
        use super::Range;

        let min = 1.0;
        let max = 0.0;

        Range::new(min, max);
    }

    #[test]
    fn test_new_infinite_range() {
        use super::Range;

        let range = Range::new_infinite();

        assert_eq!(range.min(), f64::NEG_INFINITY);
        assert_eq!(range.max(), f64::INFINITY);
    }

    #[test]
    fn test_width() {
        use super::Range;

        let min = 0.0;
        let max = 1.0;

        let range = Range::new(min, max);

        assert_eq!(range.width(), max - min);
    }

    #[test]
    fn test_contains() {
        use super::Range;

        let min = 0.0;
        let max = 1.0;

        let range = Range::new(min, max);

        assert!(range.contains(0.0));
        assert!(range.contains(0.5));
        assert!(range.contains(1.0));
        assert!(!range.contains(-0.5));
        assert!(!range.contains(1.5));
    }

    #[test]
    fn test_intersect() {
        use super::Range;

        let min = 0.0;
        let max = 1.0;

        let range = Range::new(min, max);

        assert!(range.intersect(&Range::new(0.0, 1.0)));
        assert!(range.intersect(&Range::new(0.5, 1.5)));
        assert!(range.intersect(&Range::new(-0.5, 0.5)));
        assert!(!range.intersect(&Range::new(-1.0, -0.5)));
        assert!(!range.intersect(&Range::new(1.5, 2.0)));
    }

    #[test]
    fn test_overlap() {
        use super::Range;

        let min = 0.0;
        let max = 1.0;

        let range = Range::new(min, max);

        assert_eq!(range.overlap(&Range::new(0.0, 1.0)), Some(range.clone()));
        assert_eq!(
            range.overlap(&Range::new(0.5, 1.5)),
            Some(Range::new(0.5, 1.0))
        );
        assert_eq!(
            range.overlap(&Range::new(-0.5, 0.5)),
            Some(Range::new(0.0, 0.5))
        );
        assert_eq!(range.overlap(&Range::new(-1.0, -0.5)), None);
        assert_eq!(range.overlap(&Range::new(1.5, 2.0)), None);
    }
}
