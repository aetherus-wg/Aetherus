//! Camera builder structure.

use crate::{
    fmt_report,
    geom::{Camera, Orient},
    math::{Point3, Vec3},
    core::{Build, cartesian::{X, Y}},
};
use arctk_attr::file;
use std::fmt::{Display, Error, Formatter};

/// Loadable camera structure.
#[file]
#[derive(Clone)]
pub struct CameraBuilder {
    /// Position.
    pos: Point3,
    /// Target.
    tar: Point3,
    /// Horizontal field-of-view (deg).
    fov: f64,
    /// Image resolution.
    res: [usize; 2],
    /// Optional super-sampling power.
    ss_power: Option<usize>,
}

impl CameraBuilder {
    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(
        pos: Point3,
        tar: Point3,
        fov: f64,
        res: [usize; 2],
        ss_power: Option<usize>,
    ) -> Self {
        debug_assert!(fov > 0.0);
        debug_assert!(res[X] > 0);
        debug_assert!(res[Y] > 0);
        debug_assert!(ss_power.is_none() || ss_power.unwrap() > 1);

        Self {
            pos,
            tar,
            fov,
            res,
            ss_power,
        }
    }

    /// Move the camera.
    #[inline]
    pub fn travel(&mut self, d: Vec3) {
        self.pos += d;
    }
}

impl Build for CameraBuilder {
    type Inst = Camera;

    #[inline]
    fn build(self) -> Self::Inst {
        Self::Inst::new(
            Orient::new_tar(self.pos, &self.tar),
            self.fov.to_radians(),
            self.res,
            self.ss_power.map_or(1, |ss| ss),
        )
    }
}

impl Display for CameraBuilder {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        writeln!(fmt, "...")?;
        fmt_report!(
            fmt,
            &format!("({}, {}, {})", self.pos.x(), self.pos.y(), self.pos.z()),
            "position (m)"
        );
        fmt_report!(
            fmt,
            &format!("({}, {}, {})", self.tar.x(), self.tar.y(), self.tar.z()),
            "target (m)"
        );
        fmt_report!(fmt, self.fov, "field of view (deg)");
        fmt_report!(
            fmt,
            &format!("[{} x {}]", self.res[X], self.res[Y]),
            "resolution"
        );

        let ss_power = if let Some(n) = self.ss_power {
            format!("{} sub-samples", n * n)
        } else {
            "OFF".to_owned()
        };
        fmt_report!(fmt, ss_power, "super sampling");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use tempfile::NamedTempFile;
    use super::{Camera, CameraBuilder};
    use crate::{
        fs::File,
        core::Build,
        math::linalg::{Point3, Vec3},
    };

    #[test]
    fn test_camera_builder_load() {
        let file = NamedTempFile::new().unwrap();
        let mut file2 = file.reopen().unwrap();
        file2.write_all("{ pos: [0.0, 0.0, 0.0], tar: [1.0, 0.0, 0.0], fov: 90.0, res: [640, 480] }".as_bytes()).unwrap();
        drop(file2);

        let cam: Camera = CameraBuilder::load(file.path()).unwrap().build();
        assert_eq!(*cam.pos(), Point3::new(0.0, 0.0, 0.0));
        assert_eq!(*cam.res(), [640, 480]);
        assert_eq!(cam.num_pixels(), 640 * 480);
    }

    #[test]
    fn test_camera_builder_clone() {
        // Setup the camera builder and clone it. 
        let pos = Point3::new(0., 0., 0.);
        let tar = Point3::new(-1.0, 0.0, 0.0);
        let mut build = CameraBuilder::new(pos, tar, 90.0, [640, 480], Some(2));
        build.travel(Vec3::new(1.0, 0.0, 0.0));
        let build_clone = build.clone();
        
        // Now we check to see that the properties have persisted. 
        let cam = build_clone.build();
        assert_eq!(*cam.pos(), Point3::new(1.0, 0.0, 0.0));
        assert_eq!(*cam.res(), [640, 480]);
        assert_eq!(cam.num_pixels(), 640 * 480);
        assert_eq!(cam.num_samples(), cam.num_pixels() * 4);
    }
}