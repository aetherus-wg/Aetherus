use std::ops::AddAssign;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use std::fmt::{Display, Formatter};
use ndarray::Array3;
use serde::{Deserialize, Serialize};
use crate::{
    access, 
    fmt_report,
    fs::Save, 
    geom::{Cube, Trace}, 
    math::{Point3, Vec3},
    ord::cartesian::{X, Y, Z}, 
    phys::Photon
};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "pyo3", pyclass)]
#[serde(rename_all = "lowercase")] 
pub enum OutputParameter {
    Emission,
    Energy, 
    Absorption,
    Shift,
}

#[derive(Debug, Clone)]
pub struct OutputVolume {
    boundary: Cube,
    res: [usize; 3],
    param: OutputParameter,
    voxel_size: Vec3,
    data: Array3<f64>,
}

impl OutputVolume {
    access!(boundary: Cube);
    access!(res: [usize; 3]);
    access!(param: OutputParameter);
    access!(data, data_mut: Array3<f64>);

    pub fn new(boundary: Cube, res: [usize; 3],  param: OutputParameter) -> Self {
        // Check that we don't have non-zero cell size. 
        debug_assert!(res[X] > 0);
        debug_assert!(res[Y] > 0);
        debug_assert!(res[Z] > 0);

        let mut voxel_size = boundary.widths();
        for (w, n) in voxel_size.iter_mut().zip(&res) {
            *w /= *n as f64;
        }

        Self {
            boundary,
            res,
            param,
            voxel_size,
            data: Array3::zeros(res)
        }
    }

    #[inline]
    #[must_use]
    pub fn voxel_volume(&self) -> f64 {
        self.boundary.vol() / (self.res[X] * self.res[Y] * self.res[Z]) as f64
    }

    /// Determine the total number of cells.
    #[inline]
    #[must_use]
    pub const fn num_cells(&self) -> usize {
        self.res[X] * self.res[Y] * self.res[Z]
    }

    #[inline]
    #[must_use]
    pub fn contains(&self, p: &Point3) -> bool {
        self.boundary.contains(p)
    }

    /// If the given position is contained within the grid,
    /// generate the index for the given position within the grid.
    #[inline]
    #[must_use]
    pub fn gen_index(&self, p: &Point3) -> Option<[usize; 3]> {
        self.boundary.contains(p).then(|| {
            let mins = self.boundary.mins();
            let maxs = self.boundary.maxs();

            [
                (((p.x() - mins.x()) / (maxs.x() - mins.x())) * self.res[X] as f64).floor()
                    as usize,
                (((p.y() - mins.y()) / (maxs.y() - mins.y())) * self.res[Y] as f64).floor()
                    as usize,
                (((p.z() - mins.z()) / (maxs.z() - mins.z())) * self.res[Z] as f64).floor()
                    as usize,
            ]
        })
    }

    /// If the given position is contained within the grid,
    /// generate the index and voxel for the given position within the grid.
    #[inline]
    #[must_use]
    pub fn gen_index_voxel(&self, p: &Point3) -> Option<([usize; 3], Cube)> {
        if let Some(index) = self.gen_index(p) {
            let mut min = *self.boundary.mins();
            *min.x_mut() += self.voxel_size[X] * index[X] as f64;
            *min.y_mut() += self.voxel_size[Y] * index[Y] as f64;
            *min.z_mut() += self.voxel_size[Z] * index[Z] as f64;

            let boundary = Cube::new(min, min + self.voxel_size);
            debug_assert!(boundary.contains(p));

            Some((index, boundary))
        } else {
            None
        }
    }

    /// Returns the distance to the nearest voxel boundary, if one exists. 
    #[inline]
    #[must_use]
    pub fn voxel_dist(&self, phot: &Photon) -> Option<f64> {
        let (_index, voxel) = self.gen_index_voxel(phot.ray().pos())?;
        let dist = voxel.dist(phot.ray())?;
        Some(dist)
    }
}

impl AddAssign<&Self> for OutputVolume {
    fn add_assign(&mut self, rhs: &Self) {
        debug_assert_eq!(self.boundary(), rhs.boundary());
        debug_assert_eq!(self.voxel_volume(), rhs.voxel_volume());
        debug_assert_eq!(self.param, rhs.param);

        self.data += &rhs.data
    }
}

impl Save for OutputVolume {
    #[inline]
    fn save_data(&self, path: &std::path::Path) -> Result<(), crate::err::Error> {
        (&self.data / self.voxel_volume()).save(&path)?;
        Ok(())
    }
}

impl Display for OutputVolume {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        writeln!(fmt, "...")?;
        fmt_report!(fmt, self.boundary, "boundary");
        fmt_report!(
            fmt,
            &format!("[{} x {} x {}]", self.res[X], self.res[Y], self.res[Z]),
            "resolution"
        );
        fmt_report!(
            fmt,
            &format!(
                "({}, {}, {})",
                self.voxel_size.x(),
                self.voxel_size.y(),
                self.voxel_size.z()
            ),
            "voxel size"
        );
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
        assert_eq!(outvol.voxel_volume(), 1.0);
    }
}
