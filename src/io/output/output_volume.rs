use std::ops::AddAssign;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use ndarray::Array3;
use crate::{
    access, fs::Save, geom::Cube, ord::cartesian::{X, Y, Z}
};

#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "pyo3", pyclass)]
pub enum OutputParameter {
    Energy, 
    Absorption,
    Shift,
}

#[derive(Debug)]
pub struct OutputVolume {
    boundary: Cube,
    res: [usize; 3],
    param: OutputParameter,
    data: Array3<f64>,
}

impl OutputVolume {
    access!(boundary: Cube);
    access!(res: [usize; 3]);
    access!(data, data_mut: Array3<f64>);

    pub fn new(boundary: Cube, res: [usize; 3],  param: OutputParameter) -> Self {
        // Check that we don't have non-zero cell size. 
        debug_assert!(res[X] > 0);
        debug_assert!(res[Y] > 0);
        debug_assert!(res[Z] > 0);

        Self {
            boundary,
            res,
            param,
            data: Array3::zeros(res)
        }
    }

    pub fn cell_volume(&self) -> f64 {
        self.boundary.vol() / (self.res[X] * self.res[Y] * self.res[Z]) as f64
    }
}

impl AddAssign<&Self> for OutputVolume {
    fn add_assign(&mut self, rhs: &Self) {
        debug_assert_eq!(self.boundary(), rhs.boundary());
        debug_assert_eq!(self.cell_volume(), rhs.cell_volume());
        debug_assert_eq!(self.param, rhs.param);

        self.data += &rhs.data
    }
}

impl Save for OutputVolume {
    #[inline]
    fn save_data(&self, path: &std::path::Path) -> Result<(), crate::err::Error> {
        (&self.data / self.cell_volume()).save(&path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::math::Point3;
    use super::*;

    #[test]
    fn new_test() {
        let boundary = Cube::new(Point3::new(0.0,0.0,0.0), Point3::new(10.0,10.0,10.0));
        let outvol = OutputVolume::new(boundary, [10, 10, 10], OutputParameter::Energy);
        assert_eq!(outvol.cell_volume(), 1.0);
    }
}