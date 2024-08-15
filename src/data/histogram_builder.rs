use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use crate::{
    fmt_report,
    data::Histogram,
};


#[derive(Debug, Serialize, Deserialize)]
pub struct HistogramBuilder {
    min: f64,
    max: f64,
    bins: usize,
}

impl HistogramBuilder {
    pub fn build(&self) -> Histogram {
        Histogram::new(self.min, self.max, self.bins)
    }
}

impl Display for HistogramBuilder {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        fmt_report!(fmt, self.min, "min");
        fmt_report!(fmt, self.min, "max");
        fmt_report!(fmt, self.bins, "bins");
        Ok(())
    }
}