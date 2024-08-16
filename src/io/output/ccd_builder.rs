use std::fmt::{Display, Formatter};
use serde::{Serialize, Deserialize};
use ndarray::Array3;
use crate::{
    fmt_report, 
    ord::cartesian::{X, Y}
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CcdBuilder {
    res: [usize; 2],
    bins: usize,
}

impl CcdBuilder {
    pub fn build(&self) -> Array3<f64> {
        Array3::zeros([self.res[X], self.res[Y], self.bins])
    }
}

impl Display for CcdBuilder {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        fmt_report!(fmt, format!("{} x {}", self.res[0], self.res[1]), "resolution");
        fmt_report!(fmt, self.bins, "bins");
        Ok(())
    }
}