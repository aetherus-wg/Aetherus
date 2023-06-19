//! Regular-Cartesian grid builder.

use crate::{
    access, fmt_report,
    geom::{Cube, Grid},
    ord::{Build, X, Y, Z},
};
use arctk_attr::file;
use std::fmt::{Display, Formatter};

/// Grid builder.
#[file]
#[derive(Clone)]
pub struct GridBuilder {
    /// Boundary.
    boundary: Cube,
    /// Resolution.
    res: [usize; 3],
}

impl GridBuilder {
    access!(boundary: Cube);
    access!(res: [usize; 3]);

    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(boundary: Cube, res: [usize; 3]) -> Self {
        debug_assert!(res[X] > 0);
        debug_assert!(res[Y] > 0);
        debug_assert!(res[Z] > 0);

        Self { boundary, res }
    }

    /// Determine the total number of cells.
    #[inline]
    #[must_use]
    pub const fn num_cells(&self) -> usize {
        self.res[X] * self.res[Y] * self.res[Z]
    }
}

impl Build for GridBuilder {
    type Inst = Grid;

    #[inline]
    fn build(self) -> Grid {
        Grid::new(self.boundary, self.res)
    }
}

impl Display for GridBuilder {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        writeln!(fmt, "...")?;
        fmt_report!(fmt, self.boundary, "boundary");
        fmt_report!(
            fmt,
            &format!("[{} x {} x {}]", self.res[X], self.res[Y], self.res[Z]),
            "resolution"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{math::{Point3, Vec3}, fs::File};
    use std::path::Path;

    #[test]
    fn new() {
        let boundary = Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));
        let res = [10, 10, 10];

        let grid = GridBuilder::new(boundary.clone(), res).build();

        assert_eq!(grid.boundary(), &boundary);
        assert_eq!(grid.res(), &res);
        assert_eq!(*grid.voxel_size(), Vec3::new(0.1, 0.1, 0.1));
    }

    #[test]
    #[should_panic]
    fn new_fail_x() {
        let boundary = Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));
        let res = [0, 10, 10];

        let _ = GridBuilder::new(boundary, res);
    }

    #[test]
    #[should_panic]
    fn new_fail_y() {
        let boundary = Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));
        let res = [10, 0, 10];

        let _ = GridBuilder::new(boundary, res);
    }

    #[test]
    #[should_panic]
    fn new_fail_z() {
        let boundary = Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));
        let res = [10, 10, 0];

        let _ = GridBuilder::new(boundary, res);
    }

    #[test]
    fn num_cells() {
        let boundary = Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));
        let res = [10, 10, 10];

        let grid = GridBuilder::new(boundary, res);
        assert_eq!(grid.num_cells(), 1000);
    }

    #[test]
    fn test_clone() {
        let boundary = Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));
        let res = [10, 10, 10];

        let gb = GridBuilder::new(boundary, res);
        let cloned = gb.clone();

        assert_eq!(gb.boundary(), cloned.boundary());
        assert_eq!(gb.res(), cloned.res());
    }

    #[test]
    fn deserialise_from_file_and_build() {
        // Write the example JSON to a file. 
        let grid_str = "{ boundary: { mins: [0.0, 0.0, 0.0], maxs: [1.0, 1.0, 1.0] }, res: [10, 10, 10] }";
        let path = Path::new("test_grid_builder.json");
        std::fs::write(path, grid_str).unwrap();

        // Read the file.
        let grid_builder = GridBuilder::load(&path).unwrap();
        let grid = grid_builder.build();

        // Check the results.
        assert_eq!(grid.boundary(), &Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0)));
        assert_eq!(grid.res(), &[10, 10, 10]);
        assert_eq!(*grid.voxel_size(), Vec3::new(0.1, 0.1, 0.1));

        // Delete the test input file.
        std::fs::remove_file(path).unwrap();
    }
}
