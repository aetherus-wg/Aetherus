use super::Histogram;
use crate::core::Real;

pub struct HistogramIterator<'a> {
    /// The `Histogram` instance we are iterating.
    hist: &'a Histogram,
    /// The current bin within the histogram.
    curr_bin: usize,
}

impl<'a> HistogramIterator<'a> {
    // Construct a new HistogramIterator.
    #[inline]
    #[must_use]
    pub fn new(hist: &'a Histogram) -> Self {
        Self { hist, curr_bin: 0 }
    }
}

impl<'a> Iterator for HistogramIterator<'a> {
    type Item = (Real, Real);

    fn next(&mut self) -> Option<Self::Item> {
        let nbin = self.hist.binner().bins();

        if self.curr_bin < nbin{
            let bin = self.hist.binner().range().min() + (self.curr_bin as Real * self.hist.binner().bin_width());
            let count = self.hist.counts()[self.curr_bin];
            
            self.curr_bin += 1;

            Some((bin, count))
        } else {
            self.curr_bin += 1;
            None
        }
    }
}