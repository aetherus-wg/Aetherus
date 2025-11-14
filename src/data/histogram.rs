//! Histogram implementation.

use crate::{
    access,
    data::HistogramIterator,
    err::Error,
    fmt_report,
    fs::Save,
    tools::{Binner, Range},
};
use ndarray::Array1;
use std::{
    fmt::{Display, Formatter},
    fs::File,
    io::Write,
    ops::AddAssign,
    path::Path,
};

/// Static range, constant bin width, Histogram.
#[derive(Clone)]
pub struct Histogram {
    /// Binner.
    binner: Binner,
    /// Count data.
    counts: Array1<f64>,
}

impl Histogram {
    access!(binner: Binner);
    access!(counts: Array1<f64>);

    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(min: f64, max: f64, bins: usize) -> Self {
        debug_assert!(min < max);
        debug_assert!(bins > 0);

        Self {
            binner: Binner::new(Range::new(min, max), bins),
            counts: Array1::zeros(bins as usize),
        }
    }

    /// Construct a new instance using a range.
    #[inline]
    #[must_use]
    pub fn new_range(range: Range, bins: usize) -> Self {
        Self {
            binner: Binner::new(range, bins),
            counts: Array1::zeros(bins as usize),
        }
    }

    /// Increment the bin corresponding to x by unity.
    #[inline]
    pub fn collect(&mut self, x: f64) {
        debug_assert!(self.binner.range().contains(x));

        let index = self.binner.bin(x);
        self.counts[index] += 1.0;
    }

    /// Increment the bin corresponding to x by a given weight.
    #[inline]
    pub fn collect_weight(&mut self, x: f64, weight: f64) {
        debug_assert!(self.binner.range().contains(x));
        debug_assert!(weight > 0.0);

        let index = self.binner.bin(x);
        self.counts[index] += weight;
    }

    /// Increment the bin corresponding to x by unity if x is contained within the range.
    #[inline]
    pub fn try_collect(&mut self, x: f64) {
        if let Some(index) = self.binner.try_bin(x) {
            self.counts[index] += 1.0;
        }
    }

    /// Increment the bin corresponding to x by unity if x is contained within the range.
    #[inline]
    pub fn try_collect_weight(&mut self, x: f64, weight: f64) {
        if let Some(index) = self.binner.try_bin(x) {
            self.counts[index] += weight;
        }
    }

    #[inline]
    pub fn iter(&self) -> HistogramIterator<'_> {
        HistogramIterator::new(self)
    }
}

impl AddAssign<&Self> for Histogram {
    #[inline]
    fn add_assign(&mut self, rhs: &Self) {
        debug_assert!(self.binner == rhs.binner);
        debug_assert!(self.counts.len() == rhs.counts.len());

        self.counts += &rhs.counts;
    }
}

impl Save for Histogram {
    #[inline]
    fn save_data(&self, path: &Path) -> Result<(), Error> {
        let mut file = File::create(path)?;

        let mut center = self.binner.range().min();
        let delta = self.binner.range().width() / (self.counts.len() - 1) as f64;
        for count in &self.counts {
            center += delta;
            writeln!(file, "{:>32}, {:<32}", center, count)?;
        }

        Ok(())
    }
}

impl Display for Histogram {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        fmt_report!(fmt, self.binner, "binner");
        fmt_report!(fmt, self.counts.sum(), "total counts");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::Histogram;
    use crate::{tools::Range, fs::Save};
    use tempfile::NamedTempFile;

    #[test]
    fn test_new_range() {
        let range = Range::new(0.0, 1.0);
        let mut hist = Histogram::new_range(range, 10);
        // Try collecting something in the range.
        hist.try_collect(0.55);
        // Now try collecting something outputs of the range.
        hist.try_collect(2.0);
        hist.try_collect(-1.0);

        // Check that we only have one sample in the bin.
        assert_eq!(hist.iter().map(|(_, count)| count).sum::<f64>(), 1.0);

        // Check that it is binned correctly.
        hist.iter().for_each(|(bin, count)| assert_eq!(count, if bin == 0.5 { 1.0 } else {0.0}));
    }

    #[test]
    fn test_clone() {
        let mut hist = Histogram::new(0.0, 1.0, 10);
        // Try collecting something in the range.
        hist.collect(0.25);
        hist.collect(0.55);
        hist.collect(0.75);
        let hist_clone = hist.clone();

        // Check that we only have one sample in the bin.
        assert_eq!(hist_clone.iter().map(|(_, count)| count).sum::<f64>(), 3.0);

        // Check that it is binned correctly - we do the weird comparison here to avoid rounding problems with larger bins.
        hist_clone.iter().for_each(|(bin, count)| assert_eq!(count, if (bin - 0.2).abs() < 1E-8 || (bin - 0.5).abs() < 1E-8 || (bin - 0.7).abs() < 1E-8 { 1.0 } else {0.0}));
    }

    #[test]
    fn test_try_collect() {
        let mut hist = Histogram::new(0.0, 1.0, 10);
        // Try collecting something in the range.
        hist.try_collect(0.55);
        // Now try collecting something outputs of the range.
        hist.try_collect(2.0);
        hist.try_collect(-1.0);

        // Check that we only have one sample in the bin.
        assert_eq!(hist.iter().map(|(_, count)| count).sum::<f64>(), 1.0);

        // Check that it is binned correctly.
        hist.iter().for_each(|(bin, count)| assert_eq!(count, if bin == 0.5 { 1.0 } else {0.0}));
    }

    #[test]
    fn test_collect_weight() {
        let mut hist = Histogram::new(0.0, 1.0, 10);
        // Try collecting something in the range.
        hist.collect_weight(0.55, 0.5);

        // Check that we only have one sample, weighted by the correct weight.
        assert_eq!(hist.iter().map(|(_, count)| count).sum::<f64>(), 0.5);

        // Check that it is binned correctly.
        hist.iter().for_each(|(bin, count)| assert_eq!(count, if bin == 0.5 { 0.5 } else {0.0}));
    }

    #[test]
    fn test_try_collect_weight() {
        let mut hist = Histogram::new(0.0, 1.0, 10);
        // Try collecting something in the range.
        hist.try_collect_weight(0.55, 0.5);
        // Now try collecting something outputs of the range.
        hist.try_collect_weight(2.0, 0.5);
        hist.try_collect_weight(-1.0, 0.5);

        // Check that we only have one sample, weighted by the correct weight.
        assert_eq!(hist.iter().map(|(_, count)| count).sum::<f64>(), 0.5);

        // Check that it is binned correctly.
        hist.iter().for_each(|(bin, count)| assert_eq!(count, if bin == 0.5 { 0.5 } else {0.0}));
    }

    #[test]
    fn test_add_assign() {
        let mut hist1 = Histogram::new(0.0, 1.0, 10);
        let mut hist2 = Histogram::new(0.0, 1.0, 10);
        // Try collecting something in the range.
        hist1.collect(0.25);
        hist1.collect(0.55);
        hist1.collect(0.75);
        // Populate the second histogram within the valid range.
        hist2.collect(0.15);
        hist2.collect(0.55);
        hist2.collect(0.75);

        hist1 += &hist2;

        // Check that we only have one sample in the bin.
        assert_eq!(hist1.iter().map(|(_, count)| count).sum::<f64>(), 6.0);

        // Check that it is binned correctly - we do the weird comparison here to avoid rounding problems with larger bins.
        hist1.iter().for_each(|(bin, count)| assert_eq!(count, if (bin - 0.1).abs() < 1E-8 || (bin - 0.2).abs() < 1E-8 { 1.0 } else if (bin - 0.5).abs() < 1E-8 || (bin - 0.7).abs() < 1E-8 {2.0 } else {0.0}));
    }

    #[test]
    fn test_save() {
        let mut hist = Histogram::new(0.0, 1.0, 10);
        // Try collecting something in the range.
        hist.collect(0.25);
        hist.collect(0.55);
        hist.collect(0.75);

        let file = NamedTempFile::new().unwrap();
        let res = hist.save_data(file.path());
        assert!(res.is_ok());

        let mut fileop = file.reopen().unwrap();
        let mut buf = String::new();
        assert!(fileop.read_to_string(&mut buf).is_ok());
        assert_eq!(buf.lines().count(), 10);
    }
}
