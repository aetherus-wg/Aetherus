use std::ops::AddAssign;

use ndarray::Array2;
use serde::{Deserialize, Serialize};
use std::fmt;
use crate::{
    fs::Save,
    math::{Point2, Point3},
    ord::cartesian::{X, Y, Z},
    io::output::OutputVolume,
};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "pyo3", pyclass)]
#[serde(rename_all = "lowercase")] 
pub enum AxisAlignedPlane {
    XY,
    XZ,
    YZ,
}

impl fmt::Display for AxisAlignedPlane {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let plane_str = match self {
            AxisAlignedPlane::XY => "XY",
            AxisAlignedPlane::XZ => "XZ",
            AxisAlignedPlane::YZ => "YZ",
        };
        write!(f, "{}", plane_str)
    }
}

impl AxisAlignedPlane {
    pub fn project_onto_plane(&self, p: &Point3) -> (f64, f64) {
        match *self {
            Self::XY => (p.x(), p.y()),
            Self::XZ => (p.x(), p.z()),
            Self::YZ => (p.y(), p.z()),
        }
    }

    /// Calculates the pixel area of the projected 2D plane from a given volume. 
    pub fn projected_pix_area(&self, v: &OutputVolume) -> f64 {
        v.voxel_volume() / self.hyperspectral_bin_size(v)
    }

    pub fn hyperspectral_bin_size(&self, v: &OutputVolume) -> f64 {
        match *self {
            Self::XY => v.boundary().widths()[Z] / v.res()[Z] as f64,
            Self::XZ => v.boundary().widths()[Y] / v.res()[Y] as f64,
            Self::YZ => v.boundary().widths()[X] / v.res()[X] as f64,
        }
    }
}



#[derive(Debug, Clone)]
pub struct OutputPlane {
    mins: Point2,
    maxs: Point2,
    res: [usize; 2],
    data: Array2<f64>,
    plane: AxisAlignedPlane
}

impl OutputPlane {
    pub fn new(mins: Point2, maxs: Point2, res: [usize; 2], plane: AxisAlignedPlane) -> Self {
        debug_assert!(res[X] > 0);
        debug_assert!(res[Y] > 0);
        
        Self {
            mins, 
            maxs,
            res,
            data: Array2::zeros(res),
            plane
        }
    }

    pub fn xlen(&self) -> f64 {
        self.maxs.x() - self.mins.x()
    }

    pub fn ylen(&self) -> f64 {
        self.maxs.y() - self.mins.y()
    }

    pub fn dx(&self) -> f64 {
        self.xlen() /  self.res[X] as f64
    }

    pub fn dy(&self) -> f64 {
        self.ylen() / self.res[Y] as f64
    }

    pub fn area(&self) -> f64 {
        (self.maxs.x() - self.mins.x()) * (self.maxs.y() - self.mins.y())
    }

    pub fn pix_area(&self) -> f64 {
        self.area() / (self.res[X] * self.res[Y]) as f64
    }

    pub fn index_for_coord(&self, x: f64, y:f64) -> Option<(usize, usize)> {
        if x < self.mins[X] { return None; }
        let xoff = x - self.mins[X] as f64;
        if y < self.mins[Y] { return None; }
        let yoff = y - self.mins[Y] as f64;

        if xoff < self.xlen() && yoff < self.ylen() {
            let i = (xoff / self.dx()).floor() as usize;
            let j = (yoff / self.dy()).floor() as usize;
            Some((i, j))
        } else {
            None
        }
    }

    pub fn at(&self, x: f64, y: f64) -> Option<&f64> {  
        let (i, j) = self.index_for_coord(x, y)?;
        Some(&self.data[[i, j]])
    }

    pub fn at_mut(&mut self, x: f64, y: f64) -> Option<&mut f64> {  
        let (i, j) = self.index_for_coord(x, y)?;
        Some(&mut self.data[[i, j]])
    }

    pub fn project(&self, p: &Point3) -> (f64, f64) {
        self.plane.project_onto_plane(p)
    }
}

impl AddAssign<&Self> for OutputPlane {
    fn add_assign(&mut self, rhs: &Self) {
        debug_assert_eq!(self.res, rhs.res);
        
        self.data += &rhs.data;
    }
}

impl Save for OutputPlane {
    fn save_data(&self, path: &std::path::Path) -> Result<(), crate::err::Error> {
        self.data.save(&path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::geom::Cube;
    use super::*;

    #[test]
    fn test_data_allocation() {
        let mins = Point2::new(0.0, 0.0);
        let maxs = Point2::new(10.0, 10.0);
        let res = [100, 100];
        let plane = AxisAlignedPlane::XY;
        let output_plane = OutputPlane::new(mins, maxs, res, plane);
        
        // Check that the data array is correctly allocated with the specified resolution
        assert_eq!(output_plane.data.shape(), &[100, 100]);
        assert_eq!(output_plane.data.len(), 10000);
    }

    // TODO: Check this test. 
    #[test]
    fn test_area() {
        let mins = Point2::new(0.0, 0.0);
        let maxs = Point2::new(10.0, 10.0);
        let res = [100, 100];
        let plane = AxisAlignedPlane::XY;
        let output_plane = OutputPlane::new(mins, maxs, res, plane);

        let expected_area = 100.0;
        let actual_area = output_plane.area();

        assert_eq!(actual_area, expected_area);
    }

    #[test]
    fn test_dx_dy() {
        let mins = Point2::new(0.0, 0.0);
        let maxs = Point2::new(10.0, 5.0);
        let res = [100, 100];
        let plane = AxisAlignedPlane::XY;
        let output_plane = OutputPlane::new(mins, maxs, res, plane);

        // Check the result based on the dimensions. 
        assert_eq!(output_plane.dx(), 0.1);
        assert_eq!(output_plane.dy(), 0.05);
    }

    // TODO: Check this test. 
    #[test]
    fn test_pix_area() {
        let mins = Point2::new(0.0, 0.0);
        let maxs = Point2::new(10.0, 10.0);
        let res = [100, 100];
        let plane = AxisAlignedPlane::XY;
        let output_plane = OutputPlane::new( mins, maxs, res, plane);
        let expected_pix_area = 0.01;
        let actual_pix_area = output_plane.pix_area();
        assert_eq!(actual_pix_area, expected_pix_area);
    }
    
    #[test]
    fn test_index_for_coord() {
        let mins = Point2::new(0.0, 0.0);
        let maxs = Point2::new(10.0, 10.0);
        let res = [100, 100];
        let plane = AxisAlignedPlane::XY;
        let output_plane = OutputPlane::new(mins, maxs, res, plane);

        // Check the pixel indices
        assert_eq!(output_plane.index_for_coord(5.0, 5.0), Some((50, 50)));
        assert_eq!(output_plane.index_for_coord(7.0, 7.0), Some((70, 70)));
        assert_eq!(output_plane.index_for_coord(9.0, 9.0), Some((90, 90)));
        assert_eq!(output_plane.index_for_coord(0.0, 0.0), Some((0, 0)));
        assert_eq!(output_plane.index_for_coord(10.0, 10.0), None); // Outside the grid, outer extremity. 
    }

    #[test]
    fn test_index_for_coord_outside_grid() {
        let mins = Point2::new(0.0, 0.0);
        let maxs = Point2::new(10.0, 10.0);
        let res = [100, 100];
        let plane = AxisAlignedPlane::XY;
        let output_plane = OutputPlane::new(mins, maxs, res, plane);

        // Test coordinates outside of the grid
        assert_eq!(output_plane.index_for_coord(-1.0, 5.0), None);
        assert_eq!(output_plane.index_for_coord(5.0, -1.0), None);
        assert_eq!(output_plane.index_for_coord(11.0, 5.0), None);
        assert_eq!(output_plane.index_for_coord(5.0, 11.0), None);
    }

    #[test]
    #[ignore]
    fn test_edit_value() {
        let mins = Point2::new(0.0, 0.0);
        let maxs = Point2::new(10.0, 10.0);
        let res = [100, 100];
        let plane = AxisAlignedPlane::XY;
        let mut output_plane = OutputPlane::new(mins, maxs, res, plane);

        // Set initial values
        *output_plane.at_mut(5.0, 5.0).unwrap() = 1.0;
        *output_plane.at_mut(7.0, 7.0).unwrap() = 2.0;
        *output_plane.at_mut(9.0, 9.0).unwrap() = 3.0;

        // Check that the values have been updated
        assert_eq!(output_plane.at(5.0, 5.0), Some(&1.0));
        assert_eq!(output_plane.at(7.0, 7.0), Some(&2.0));
        assert_eq!(output_plane.at(9.0, 9.0), Some(&3.0));
    }

    #[test]
    fn test_projected_pix_area() {
        let mins = Point3::new(0., 0., 0.);
        let maxs = Point3::new(3., 2., 1.);
        let output_volume = OutputVolume::new(
            Cube::new(mins, maxs), 
            [10, 10, 10], 
            crate::io::output::OutputParameter::Hyperspectral,
        );

        assert_eq!(AxisAlignedPlane::XY.projected_pix_area(&output_volume), 0.06);
        assert_eq!(AxisAlignedPlane::XZ.projected_pix_area(&output_volume), 0.03);
        assert_eq!(AxisAlignedPlane::YZ.projected_pix_area(&output_volume), 0.02);
    }
}