//! Axis-aligned-bounding-box implementation.

use crate::{
    access, fmt_report,
    geom::{Collide, Mesh, Ray, Side, Trace},
    math::{Dir3, Point3, Vec3},
    ord::cartesian::{X, Y, Z},
    tools::Range,
};
use arctk_attr::file;
use rand::Rng;
use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
};

/// Axis-aligned bounding box geometry.
/// Used for spatial partitioning.
#[file]
#[derive(Clone, PartialEq)]
pub struct Cube {
    /// Minimum bound.
    mins: Point3,
    /// Maximum bound.
    maxs: Point3,
}

impl Cube {
    access!(mins: Point3);
    access!(maxs: Point3);

    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(mins: Point3, maxs: Point3) -> Self {
        debug_assert!(mins < maxs);

        Self { mins, maxs }
    }

    /// Construct a new axis-aligned bounding box centred on a given point with given half widths.
    #[inline]
    #[must_use]
    pub fn new_centred(centre: &Point3, hws: &Vec3) -> Self {
        debug_assert!(hws.iter().all(|x| *x > 0.0));

        Self::new(centre - hws, centre + hws)
    }

    /// Initialise the boundary encompassing all of the mesh vertices.
    #[inline]
    #[must_use]
    pub fn new_shrink(surfs: &[Mesh]) -> Self {
        let mut mins = None;
        let mut maxs = None;

        for mesh in surfs {
            let (mesh_mins, mesh_maxs) = mesh.boundary().mins_maxs();

            if mins.is_none() {
                mins = Some(mesh_mins);
            } else {
                for (grid_min, mesh_min) in mins.as_mut().unwrap().iter_mut().zip(mesh_mins.iter())
                {
                    if mesh_min < grid_min {
                        *grid_min = *mesh_min;
                    }
                }
            }

            if maxs.is_none() {
                maxs = Some(mesh_maxs);
            } else {
                for (grid_max, mesh_max) in maxs.as_mut().unwrap().iter_mut().zip(mesh_maxs.iter())
                {
                    if mesh_max > grid_max {
                        *grid_max = *mesh_max;
                    }
                }
            }
        }

        Self::new(mins.unwrap(), maxs.unwrap())
    }

    /// Get mins and maxs together.
    #[inline]
    #[must_use]
    pub const fn mins_maxs(&self) -> (Point3, Point3) {
        (self.mins, self.maxs)
    }

    /// Calculate the widths.
    #[inline]
    #[must_use]
    pub fn widths(&self) -> Vec3 {
        self.maxs - self.mins
    }

    /// Calculate the half-widths.
    #[inline]
    #[must_use]
    pub fn half_widths(&self) -> Vec3 {
        self.widths() * 0.5
    }

    /// Calculate the centre position.
    #[inline]
    #[must_use]
    pub fn centre(&self) -> Point3 {
        nalgebra::center(&self.mins.data(), &self.maxs.data()).into()
    }

    /// Calculate the surface area.
    #[inline]
    #[must_use]
    pub fn area(&self) -> f64 {
        let ws = self.widths();
        2.0 * ws
            .z()
            .mul_add(ws.x(), ws.x().mul_add(ws.y(), ws.y() * ws.z()))
    }

    /// Calculate the volume.
    #[inline]
    #[must_use]
    pub fn vol(&self) -> f64 {
        let ws = self.widths();
        ws.x() * ws.y() * ws.z()
    }

    /// Determine if the given point if contained.
    #[inline]
    #[must_use]
    pub fn contains(&self, p: &Point3) -> bool {
        p >= &self.mins && p <= &self.maxs
    }

    /// Shrink the aabb by a fraction of its lengths, maintaining the central position.
    #[inline]
    pub fn shrink(&mut self, f: f64) {
        debug_assert!(f > 0.0);
        debug_assert!(f < 1.0);

        let delta = self.half_widths() * f;

        self.mins += delta;
        self.maxs -= delta;
    }

    /// Expand the aabb by a fraction of its lengths, maintaining the central position.
    #[inline]
    pub fn expand(&mut self, f: f64) {
        debug_assert!(f > 0.0);

        let delta = self.half_widths() * f;

        self.mins -= delta;
        self.maxs += delta;
    }

    /// Determine the intersection distances along a ray's direction.
    #[inline]
    #[must_use]
    fn intersections(&self, ray: &Ray) -> (f64, f64) {
        // TODO: Precompute 1/ray.dir() as it's used at every Voxel transition and f64 div
        // takes much longer thatn f64 mul
        let t_0: Vec<_> = self
            .mins
            .iter()
            .zip(ray.pos().iter().zip(ray.dir().iter()))
            .map(|(m, (p, d))| (m - p) / d)
            .map(|x| if x== -0.0 {0.0} else {x}) // Handle negative zero case
            .collect();

        let t_1: Vec<_> = self
            .maxs
            .iter()
            .zip(ray.pos().iter().zip(ray.dir().iter()))
            .map(|(m, (p, d))| (m - p) / d)
            .map(|x| if x== -0.0 {0.0} else {x}) // Handle negative zero case
            .collect();

        let t_min = t_0
            .iter()
            .zip(t_1.iter())
            .map(|(a, b)| a.min(*b))
            .max_by(|a, b| {
                if a < b {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            })
            .unwrap();

        let t_max = t_0
            .iter()
            .zip(t_1.iter())
            .map(|(a, b)| a.max(*b))
            .min_by(|a, b| {
                if a < b {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            })
            .unwrap();

        (t_min, t_max)
    }

    /// Generate a random position within the cube's volume.
    #[inline]
    #[must_use]
    pub fn rand_pos<R: Rng>(&self, rng: &mut R) -> Point3 {
        let widths = self.widths();

        let x = self.mins.x() + rng.gen_range(0.0..widths.x());
        let y = self.mins.y() + rng.gen_range(0.0..widths.y());
        let z = self.mins.z() + rng.gen_range(0.0..widths.z());

        Point3::new(x, y, z)
    }

    /// Generate a uniformly indexed position within the cube's volume.
    #[inline]
    #[must_use]
    pub fn uniform_pos(&self, res: &[usize; 3], index: &[usize; 3]) -> Point3 {
        debug_assert!(res[X] > 0);
        debug_assert!(res[Y] > 0);
        debug_assert!(res[Z] > 0);
        debug_assert!(res[X] > index[X]);
        debug_assert!(res[Y] > index[Y]);
        debug_assert!(res[Z] > index[Z]);

        let ws = self.widths();
        let half_deltas = Point3::new(
            ws.x() / (res[X] * 2) as f64,
            ws.y() / (res[Y] * 2) as f64,
            ws.z() / (res[Z] * 2) as f64,
        );

        let x = half_deltas
            .x()
            .mul_add(((index[X] * 2) + 1) as f64, self.mins.x());
        let y = half_deltas
            .y()
            .mul_add(((index[Y] * 2) + 1) as f64, self.mins.y());
        let z = half_deltas
            .z()
            .mul_add(((index[Z] * 2) + 1) as f64, self.mins.z());

        Point3::new(x, y, z)
    }
}

impl Collide for Cube {
    #[inline]
    fn overlap(&self, aabb: &Cube) -> bool {
        self.mins <= aabb.maxs && self.maxs >= aabb.mins
    }
}

impl Trace for Cube {
    #[inline]
    fn hit(&self, ray: &Ray) -> bool {
        let (t_min, t_max) = self.intersections(ray);

        !(t_max <= 0.0 || t_min > t_max)
    }

    #[inline]
    fn dist(&self, ray: &Ray) -> Option<f64> {
        let (t_min, t_max) = self.intersections(ray);

        if t_max < 0.0 || t_min > t_max {
            return None;
        }

        if t_min > 0.0 {
            return Some(t_min);
        }

        Some(t_max)
    }

    #[inline]
    fn dist_side(&self, ray: &Ray) -> Option<(f64, Side)> {
        if let Some(dist) = self.dist(ray) {
            let hit = *ray.pos() + (dist * ray.dir());
            let relative = hit - self.centre();

            let xy = relative.y() / relative.x();
            let zy = relative.z() / relative.y();

            let unit_range = Range::new(-1.0, 1.0);
            let norm = Dir3::from(if unit_range.contains(xy) {
                Vec3::new(1.0_f64.copysign(relative.x()), 0.0, 0.0)
            } else if unit_range.contains(zy) {
                Vec3::new(0.0, 1.0_f64.copysign(relative.y()), 0.0)
            } else {
                Vec3::new(0.0, 0.0, 1.0_f64.copysign(relative.z()))
            });

            return Some((dist, Side::new(ray.dir(), norm)));
        }

        None
    }
}

impl Display for Cube {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        writeln!(fmt, "...")?;
        fmt_report!(
            fmt,
            &format!("({}, {}, {})", self.mins.x(), self.mins.y(), self.mins.z()),
            "mins (m)"
        );
        fmt_report!(
            fmt,
            &format!("({}, {}, {})", self.maxs.x(), self.maxs.y(), self.maxs.z()),
            "maxs (m)"
        );
        let c = self.centre();
        fmt_report!(
            fmt,
            &format!("({}, {}, {})", c.x(), c.y(), c.z()),
            "center (m)"
        );
        fmt_report!(fmt, self.area(), "area (m^2)");
        fmt_report!(fmt, self.vol(), "volume (m^3)");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::geom::{Triangle, SmoothTriangle};
    use assert_approx_eq::assert_approx_eq;
    use super::*;

    #[test]
    fn test_cube_new() {
        let cube = Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(2.0, 2.0, 2.0));
        assert_eq!(cube.mins, Point3::new(0.0, 0.0, 0.0));
        assert_eq!(cube.maxs, Point3::new(2.0, 2.0, 2.0));
        assert_eq!(cube.centre(), Point3::new(1.0, 1.0, 1.0));
        assert_eq!(cube.half_widths(), Vec3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn test_new_centre() {
        let cube = Cube::new_centred(&Point3::new(1.0, 1.0, 1.0), &Vec3::new(1.0, 1.0, 1.0));

        // Try this using the alternate method so that we can test this in the process.
        let (mins, maxs) = cube.mins_maxs();
        assert_eq!(mins, Point3::new(0.0, 0.0, 0.0));
        assert_eq!(maxs, Point3::new(2.0, 2.0, 2.0));
        assert_eq!(cube.centre(), Point3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn test_new_shrink() {
        // Make a single upward facing triangle to emit from.
        let tris = vec![ SmoothTriangle::new(
            Triangle::new([
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(1.0, 0.0, 0.0),
                Point3::new(0.0, 1.0, 0.0),
        ]),
            [Dir3::new(0.0, 0.0, 1.0), Dir3::new(0.0, 0.0, 1.0), Dir3::new(0.0, 0.0, 1.0)],
        )];
        let mesh = Mesh::new(tris);

        let cube = Cube::new_shrink(&[mesh]);
        let (mins, maxs) = cube.mins_maxs();
        assert_eq!(mins, Point3::new(-1.0e-6, -1.0e-6, -1.0e-6));
        assert_eq!(maxs, Point3::new(1.000001, 1.000001, 1e-6));

        assert_approx_eq!(cube.centre().x(), 0.5);
        assert_approx_eq!(cube.centre().y(), 0.5);
        assert_approx_eq!(cube.centre().z(), 0.0);
    }

    #[test]
    fn test_cube_area() {
        let cube = Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));
        assert_eq!(cube.area(), 6.0);
    }

    #[test]
    fn test_cube_vol() {
        let cube = Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));
        assert_eq!(cube.vol(), 1.0);
    }

    #[test]
    fn test_shrink_expand() {
        let mut cube = Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(2.0, 2.0, 2.0));
        assert_eq!(cube.mins, Point3::new(0.0, 0.0, 0.0));
        assert_eq!(cube.maxs, Point3::new(2.0, 2.0, 2.0));
        assert_eq!(cube.centre(), Point3::new(1.0, 1.0, 1.0));

        cube.shrink(0.5);
        assert_eq!(cube.mins, Point3::new(0.5, 0.5, 0.5));
        assert_eq!(cube.maxs, Point3::new(1.5, 1.5, 1.5));
        assert_eq!(cube.centre(), Point3::new(1.0, 1.0, 1.0));

        cube.expand(0.5);
        assert_eq!(cube.mins, Point3::new(0.25, 0.25, 0.25));
        assert_eq!(cube.maxs, Point3::new(1.75, 1.75, 1.75));
        assert_eq!(cube.centre(), Point3::new(1.0, 1.0, 1.0));
    }

    /// A unit test to check the generation of random positions within the cube volume, taking 10,000 samples.
    #[test]
    fn test_random_pos() {
        let cube = Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));

        let mut rng = rand::thread_rng();
        let mut count = 0;
        let mut sum = 0.0;
        let mut sum_sq = 0.0;
        let n = 10000;
        while count < n {
            let pos = cube.rand_pos(&mut rng);
            assert!(cube.contains(&pos));
            sum += pos.x();
            sum_sq += pos.x() * pos.x();
            count += 1;
        }

        let mean = sum / n as f64;
        let var = sum_sq / n as f64 - mean * mean;
        assert!(mean - 0.5 < 0.01);
        assert!(var - 1.0 / 12.0 < 0.01);
    }

    #[test]
    fn test_intersection_hit() {
        let cube = Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(2.0, 2.0, 2.0));
        let ray = Ray::new(Point3::new(1.0, 1.0, -1.0), Dir3::new(0.0, 0.0, 1.0));
        let (t_min, t_max) = cube.intersections(&ray);

        assert_eq!(t_min, 1.0);
        assert_eq!(t_max, 3.0);
    }

    #[test]
    fn test_intersection_miss() {
        let cube = Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(2.0, 2.0, 2.0));
        let ray = Ray::new(Point3::new(1.0, 1.0, -1.0), Dir3::new(0.0, 0.0, -1.0));
        let (t_min, t_max) = cube.intersections(&ray);

        // The ray is pointing away from the cube, so there should be no intersections, hence no positive solutions for distance.
        assert!(t_min < 0.0);
        assert!(t_max < 0.0);
    }

    #[test]
    fn test_cube_trace_inside() {
        let cube = Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));
        let ray = Ray::new(Point3::new(0.5, 0.5, 2.0), Dir3::new(0.0, 0.0, -1.0));

        // Determine if a Ray hit occurs -- it should.
        let will_hit = cube.hit(&ray);
        assert_eq!(will_hit, true);

        let hit = cube.dist_side(&ray);
        assert!(hit.is_some());
        let (dist, side) = hit.unwrap();
        assert_eq!(dist, 1.0);
        assert_eq!(side, Side::Inside(Dir3::new(-1.0, -0.0, -0.0)));
    }

    #[test]
    #[ignore = "This case is currently not handled correctly."]
    fn test_cube_trace_outside() {
        let cube = Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));
        let ray = Ray::new(Point3::new(0.5, 0.5, -2.0), Dir3::new(0.0, 0.0, 1.0));

        // Determine if a Ray hit occurs -- it should.
        let will_hit = cube.hit(&ray);
        assert_eq!(will_hit, true);

        let hit = cube.dist_side(&ray);
        assert!(hit.is_some());
        let (dist, side) = hit.unwrap();
        assert_eq!(dist, 1.0);
        assert_eq!(side, Side::Outside(Dir3::new(-1.0, -0.0, -0.0)));
    }

    #[test]
    fn test_cube_trace_miss() {
        let cube = Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));
        let ray = Ray::new(Point3::new(0.5, 0.5, 2.0), Dir3::new(0.0, 0.0, 1.0));

        // Determine if a Ray hit occurs -- it should not.
        let will_hit = cube.hit(&ray);
        assert_eq!(will_hit, false);

        let hit = cube.dist_side(&ray);
        assert!(hit.is_none());
    }

    #[test]
    fn test_uniform_pos() {
        let cube = Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(2.0, 2.0, 2.0));
        let res = [10, 10, 10];

        // Try the lower end of the domain.
        let sample = cube.uniform_pos(&res, &[0, 0, 0]);
        assert_eq!(sample, Point3::new(0.1, 0.1, 0.1));

        // Try the upper end of the domain.
        let sample = cube.uniform_pos(&res, &[9, 9, 9]);
        assert_approx_eq!(sample.x(), 1.9);
        assert_approx_eq!(sample.y(), 1.9);
        assert_approx_eq!(sample.z(), 1.9);

        // Try the middle of the domain.
        let sample = cube.uniform_pos(&res, &[5, 5, 5]);
        assert_approx_eq!(sample.x(), 1.1);
        assert_approx_eq!(sample.y(), 1.1);
        assert_approx_eq!(sample.z(), 1.1);
    }

    #[test]
    #[should_panic]
    fn test_uniform_pos_index_too_large() {
        let cube = Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(2.0, 2.0, 2.0));
        let res = [10, 10, 10];

        // Try a point outside the domain.
        let _ = cube.uniform_pos(&res, &[10, 10, 10]);
    }

    #[test]
    #[should_panic]
    fn test_uniform_pos_res_zero() {
        let cube = Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(2.0, 2.0, 2.0));
        let res = [0, 0, 0];

        // Try to sample with zero resolution.
        let _ = cube.uniform_pos(&res, &[0, 0, 0]);
    }
}
