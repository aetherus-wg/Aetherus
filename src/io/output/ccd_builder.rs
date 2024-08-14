use serde::{Serialize, Deserialize};
use ndarray::Array3;
use crate::ord::cartesian::{X, Y};

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