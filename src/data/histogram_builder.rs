use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use crate::{
    data::Histogram, fmt_report, ord::Build
};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistogramBuilder {
    min: f64,
    max: f64,
    bins: usize,
}

impl Build for HistogramBuilder {
    type Inst = Histogram;
    type MetaInfo = ();
    fn build(self, _id: ()) -> Result<Self::Inst, crate::err::Error> {
        Ok(Histogram::new(self.min, self.max, self.bins))
    }
}

impl Display for HistogramBuilder {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        fmt_report!(fmt, self.min, "min");
        fmt_report!(fmt, self.min, "max");
        fmt_report!(fmt, self.bins, "bins");
        Ok(())
    }
}
