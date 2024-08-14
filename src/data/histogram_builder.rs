use serde::{Deserialize, Serialize};

use super::Histogram;


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