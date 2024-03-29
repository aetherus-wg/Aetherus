//! Camera structure.

use crate::{
    access, clone, fmt_report,
    geom::{Orient, Ray},
    math::{Point3, Rot3, Vec3},
    ord::{X, Y},
};
use std::fmt::{Display, Error, Formatter};

/// Tracer emission structure.
pub struct Camera {
    /// Orientation.
    orient: Orient,
    /// Rotation delta.
    half_delta_theta: f64,
    /// Resolution.
    res: [usize; 2],
    /// Super sampling power.
    ss_power: usize,
}

impl Camera {
    access!(res: [usize; 2]);
    clone!(ss_power: usize);

    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(orient: Orient, fov: f64, res: [usize; 2], ss_power: usize) -> Self {
        debug_assert!(fov > 0.0);
        debug_assert!(res[X] > 0);
        debug_assert!(res[Y] > 0);
        debug_assert!(ss_power > 0);

        let half_delta_theta = fov / ((2 * (ss_power * (res[X] - 1))) as f64);

        Self {
            orient,
            half_delta_theta,
            res,
            ss_power,
        }
    }

    /// Reference the camera's position.
    #[inline]
    #[must_use]
    pub const fn pos(&self) -> &Point3 {
        self.orient.pos()
    }

    /// Calculate the total number of samples.
    #[inline]
    #[must_use]
    pub const fn num_pixels(&self) -> usize {
        self.res[X] * self.res[Y]
    }

    /// Calculate the total number of super samples per pixel.
    #[inline]
    #[must_use]
    pub const fn num_super_samples(&self) -> usize {
        self.ss_power * self.ss_power
    }

    /// Calculate the total number of samples.
    #[inline]
    #[must_use]
    pub const fn num_samples(&self) -> usize {
        self.num_super_samples() * self.num_pixels() as usize
    }

    /// Emit a ray for the given pixel and super-sample.
    #[inline]
    #[must_use]
    pub fn emit(&self, pixel: [usize; 2], ss: [usize; 2]) -> Ray {
        debug_assert!(pixel[X] < self.res[X]);
        debug_assert!(pixel[Y] < self.res[Y]);
        debug_assert!(ss[X] < self.ss_power);
        debug_assert!(ss[Y] < self.ss_power);

        let mut theta =
            self.half_delta_theta * (1 + (2 * (ss[X] + (pixel[X] * self.ss_power)))) as f64;
        let mut phi =
            self.half_delta_theta * (1 + (2 * (ss[Y] + (pixel[Y] * self.ss_power)))) as f64;

        theta -= self.half_delta_theta * (self.res[X] * self.ss_power) as f64;
        phi -= self.half_delta_theta * (self.res[Y] * self.ss_power) as f64;

        let mut ray = self.orient.forward_ray();
        *ray.dir_mut() = Rot3::from_axis_angle(&Vec3::from(self.orient.down()), theta)
            * Rot3::from_axis_angle(&Vec3::from(*self.orient.right()), phi)
            * ray.dir();

        ray
    }
}

impl Display for Camera {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        writeln!(fmt, "...")?;
        fmt_report!(fmt, self.orient, "orientation");
        fmt_report!(fmt, self.half_delta_theta.to_degrees(), "dTheta/2 (deg)");
        fmt_report!(
            fmt,
            &format!("[{} x {}]", self.res[X], self.res[Y]),
            "resolution"
        );
        fmt_report!(fmt, self.ss_power, "super sampling power");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        geom::{CameraBuilder}, 
        math::{Point3, Vec3, Dir3},
        ord::Build,
    };
    use rand::random;

    #[test]
    fn test_build_camera() {
        let pos = Point3::new(0., 0., 0.);
        let tar = Point3::new(-1.0, 0.0, 0.0);
        let mut build = CameraBuilder::new(pos, tar, 90.0, [640, 480], Some(2));
        build.travel(Vec3::new(1.0, 0.0, 0.0));
        let cam = build.build();

        assert_eq!(*cam.pos(), Point3::new(1.0, 0.0, 0.0));
        assert_eq!(*cam.res(), [640, 480]);
        assert_eq!(cam.num_pixels(), 640 * 480);
        assert_eq!(cam.num_samples(), cam.num_pixels() * 4);
    }

    #[test]
    fn test_camera_emit() {
        let res = [640, 480];
        let pos = Point3::new(0., 0., 0.);
        let tar = Point3::new(-1.0, 0.0, 0.0);
        let mut build = CameraBuilder::new(pos, tar, 90.0, res, None);
        build.travel(Vec3::new(1.0, 0.0, 0.0));
        let cam = build.build();

        let test_dir = Dir3::new(-1.0, 0.0, 0.0);
        for _ in 0..10_000 {
            let xpix: usize = (random::<f64>() * (res[0] - 1) as f64).round() as usize;
            let ypix: usize = (random::<f64>() * (res[1] - 1) as f64).round() as usize;
            let ray = cam.emit([xpix, ypix], [0, 0]);
            assert!(ray.dir().dot(&test_dir) >= 0.5);
        }

    }
}