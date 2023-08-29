//! Regular Cartesian-grid cell scheme.

use crate::{
    access, fmt_report,
    geom::Cube,
    math::{Point3, Vec3},
    ord::{X, Y, Z},
};
use std::fmt::{Display, Formatter};

/// Regular Cartesian-grid structure.
#[derive(Clone)]
pub struct Grid {
    /// Boundary.
    boundary: Cube,
    /// Resolution.
    res: [usize; 3],
    /// Voxel size.
    voxel_size: Vec3,
}

impl Grid {
    access!(boundary: Cube);
    access!(res: [usize; 3]);
    access!(voxel_size: Vec3);

    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(boundary: Cube, res: [usize; 3]) -> Self {
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
            voxel_size,
        }
    }

    /// Calculate the voxel volume.
    #[inline]
    #[must_use]
    pub fn voxel_vol(&self) -> f64 {
        self.voxel_size.x() * self.voxel_size.y() * self.voxel_size.z()
    }

    /// Determine the total number of cells.
    #[inline]
    #[must_use]
    pub const fn num_cells(&self) -> usize {
        self.res[X] * self.res[Y] * self.res[Z]
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

    /// Generate the voxel for the given index.
    #[inline]
    #[must_use]
    pub fn gen_voxel(&self, index: &[usize; 3]) -> Cube {
        debug_assert!(index[X] < self.res[X]);
        debug_assert!(index[Y] < self.res[Y]);
        debug_assert!(index[Z] < self.res[Z]);

        let x = self
            .voxel_size
            .x()
            .mul_add(index[X] as f64, self.boundary.mins().x());
        let y = self
            .voxel_size
            .y()
            .mul_add(index[Y] as f64, self.boundary.mins().y());
        let z = self
            .voxel_size
            .z()
            .mul_add(index[Z] as f64, self.boundary.mins().z());

        let mins = Point3::new(x, y, z);

        Cube::new(mins, mins + self.voxel_size)
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
}

impl Display for Grid {
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
    use super::*;
    use crate::{
        geom::Cube,
    };

    #[test]
    fn test_new() {
        let mins = Point3::new(0.0, 0.0, 0.0);
        let maxs = Point3::new(1.0, 1.0, 1.0);
        let boundary = Cube::new(mins, maxs);
        let res = [2, 2, 2];
        let grid = Grid::new(boundary, res);

        assert_eq!(grid.boundary(), &Cube::new(mins, maxs));
        assert_eq!(grid.res(), &[2, 2, 2]);
        assert_eq!(grid.voxel_size(), &Vec3::new(0.5, 0.5, 0.5));
        assert_eq!(grid.voxel_vol(), 0.125);
        assert_eq!(grid.num_cells(), 8);
    }

    #[test]
    fn test_gen_index_voxel() {
        let mins = Point3::new(0.0, 0.0, 0.0);
        let maxs = Point3::new(1.0, 1.0, 1.0);
        let boundary = Cube::new(mins, maxs);
        let res = [2, 2, 2];
        let grid = Grid::new(boundary, res);

        let p = Point3::new(0.5, 0.5, 0.5);
        let (index, voxel) = grid.gen_index_voxel(&p).unwrap();
        assert_eq!(index, [1, 1, 1]);
        assert_eq!(voxel, Cube::new(Point3::new(0.5, 0.5, 0.5), Point3::new(1.0, 1.0, 1.0)));
    }

    #[test]
    fn test_gen_index_voxel_outside_boundary() {
        let mins = Point3::new(0.0, 0.0, 0.0);
        let maxs = Point3::new(1.0, 1.0, 1.0);
        let boundary = Cube::new(mins, maxs);
        let res = [2, 2, 2];
        let grid = Grid::new(boundary, res);

        let p = Point3::new(2.0, 2.0, 2.0);
        let result = grid.gen_index_voxel(&p);
        assert!(result.is_none());
    }

    #[test]
    fn test_gen_voxel() {
        let mins = Point3::new(0.0, 0.0, 0.0);
        let maxs = Point3::new(1.0, 1.0, 1.0);
        let boundary = Cube::new(mins, maxs);
        let res = [2, 2, 2];
        let grid = Grid::new(boundary, res);

        let index = [1, 1, 1];
        let voxel = grid.gen_voxel(&index);
        assert_eq!(index, [1, 1, 1]);
        assert_eq!(voxel, Cube::new(Point3::new(0.5, 0.5, 0.5), Point3::new(1.0, 1.0, 1.0)));
    }

    #[test]
    fn test_clone() {
        let boundary = Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));
        let res = [10, 10, 10];

        let grid = Grid::new(boundary, res);
        let cloned = grid.clone();

        assert_eq!(grid.boundary(), cloned.boundary());
        assert_eq!(grid.res(), cloned.res());
    }
}