//! Orientation structure.

use crate::{
    access, fmt_report,
    geom::Ray,
    math::{Dir3, Point3, Vec3},
};
use std::fmt::{Display, Error, Formatter};

/// # Orientation
///
/// Contains orientation information about an object.
/// The struct contains the forward, right and up directions.
#[derive(Clone, Debug, PartialEq)]
pub struct Orient {
    /// Position.
    pos: Point3,
    /// Forward direction.
    forward: Dir3,
    /// Right direction.
    right: Dir3,
    /// Up direction.
    up: Dir3,
}

impl Orient {
    access!(pos, pos_mut: Point3);
    access!(forward: Dir3);
    access!(right: Dir3);
    access!(up: Dir3);

    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(ray: Ray) -> Self {
        let (pos, forward) = ray.destruct();
        let right = if forward.z().abs() <= 0.9 {
            Dir3::from(forward.cross(&Vec3::z_axis())) // Universal up is z-axis.
        } else {
            Dir3::from(forward.cross(&Vec3::x_axis())) // If facing along z-axis, compute relative up using x-axis.
        };
        let up = Dir3::from(right.cross(&forward));

        Self {
            pos,
            forward,
            right,
            up,
        }
    }

    /// Construct with an up-direction.
    #[inline]
    #[must_use]
    pub fn new_up(ray: Ray, up: &Dir3) -> Self {
        let (pos, forward) = ray.destruct();
        let right = Dir3::from(forward.cross(up));
        let up = Dir3::from(right.cross(&forward));

        Self {
            pos,
            forward,
            right,
            up,
        }
    }

    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new_tar(pos: Point3, tar: &Point3) -> Self {
        Self::new(Ray::new(pos, Dir3::from(tar - pos)))
    }

    /// Reference the backward direction.
    #[inline]
    #[must_use]
    pub fn back(&self) -> Dir3 {
        -self.forward
    }

    /// Reference the left direction.
    #[inline]
    #[must_use]
    pub fn left(&self) -> Dir3 {
        -self.right
    }

    /// Reference the downward direction.
    #[inline]
    #[must_use]
    pub fn down(&self) -> Dir3 {
        -self.up
    }

    /// Create a forward ray.
    #[inline]
    #[must_use]
    pub fn forward_ray(&self) -> Ray {
        Ray::new(self.pos, self.forward)
    }

    /// Create a backward ray.
    #[inline]
    #[must_use]
    pub fn backward_ray(&self) -> Ray {
        Ray::new(self.pos, -self.forward)
    }

    /// Create a upward ray.
    #[inline]
    #[must_use]
    pub fn up_ray(&self) -> Ray {
        Ray::new(self.pos, self.up)
    }

    /// Create a downward ray.
    #[inline]
    #[must_use]
    pub fn down_ray(&self) -> Ray {
        Ray::new(self.pos, -self.up)
    }

    /// Create a right ray.
    #[inline]
    #[must_use]
    pub fn right_ray(&self) -> Ray {
        Ray::new(self.pos, self.right)
    }

    /// Create a left ray.
    #[inline]
    #[must_use]
    pub fn left_ray(&self) -> Ray {
        Ray::new(self.pos, -self.right)
    }
}

impl Display for Orient {
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
            &format!(
                "({}, {}, {})",
                self.forward.x(),
                self.forward.y(),
                self.forward.z()
            ),
            "forwards"
        );
        fmt_report!(
            fmt,
            &format!(
                "({}, {}, {})",
                self.right.x(),
                self.right.y(),
                self.right.z()
            ),
            "rightwards"
        );
        fmt_report!(
            fmt,
            &format!("({}, {}, {})", self.up.x(), self.up.y(), self.up.z()),
            "upwards"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Orient;
    use crate::{
        geom::Ray,
        math::{Dir3, Point3},
    };
    use assert_approx_eq::assert_approx_eq;
    use std::f64;

    /// Checks that we can initialise and pull back the correct orientation information
    /// using an Orient object.
    #[test]
    fn make_new_test() {
        let ray = Ray::new(Point3::new(0., 0., 0.), Dir3::new(1., 0., 0.));
        let orient = Orient::new(ray);

        // Check that it is in the correct position.
        assert_eq!(orient.pos(), &Point3::new(0.0, 0.0, 0.0));

        // Make sure that each of the orientations are pointing in the correct direction.
        assert_eq!(orient.forward(), &Dir3::new(1.0, 0.0, 0.0));
        assert_eq!(orient.right(), &Dir3::new(0.0, -1.0, 0.0));
        assert_eq!(orient.up(), &Dir3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn make_new_target_test() {
        let pos = Point3::new(0.0, 0.0, 0.0);
        let target = Point3::new(1.0, 1.0, 1.0);
        let orient = Orient::new_tar(pos, &target);

        // Check that it is in the correct position.
        assert_eq!(orient.pos(), &Point3::new(0.0, 0.0, 0.0));

        // Make sure that each of the orientations are pointing in the correct direction.
        assert_eq!(orient.forward(), &Dir3::new(1.0, 1.0, 1.0));

        // TODO: Check that these are correct.
        assert_approx_eq!(orient.left().x(), -1.0 / 2.0_f64.sqrt(), f64::EPSILON);
        assert_approx_eq!(orient.left().y(), 1.0 / 2.0_f64.sqrt(), f64::EPSILON);
        assert_approx_eq!(orient.left().z(), 0.0, f64::EPSILON);

        assert_approx_eq!(orient.up().x(), -1.0 / 6.0f64.sqrt(), f64::EPSILON);
        assert_approx_eq!(orient.up().y(), -1.0 / 6.0f64.sqrt(), f64::EPSILON);
        assert_approx_eq!(orient.up().z(), 1.0 / 1.50f64.sqrt(), f64::EPSILON);
    }

    #[test]
    fn directions_test() {
        let ray = Ray::new(Point3::new(0., 0., 0.), Dir3::new(1., 0., 0.));
        let orient = Orient::new(ray);

        // Check that it is in the correct position.
        assert_eq!(orient.pos(), &Point3::new(0.0, 0.0, 0.0));

        // Make sure that each of the orientations are pointing in the correct direction.
        assert_eq!(orient.forward(), &Dir3::new(1.0, 0.0, 0.0));
        assert_eq!(orient.right(), &Dir3::new(0.0, -1.0, 0.0));
        assert_eq!(orient.up(), &Dir3::new(0.0, 0.0, 1.0));

        // Now check that inverse directions return the inverse of their calculated counterparts. 
        assert_eq!(orient.back(), Dir3::new(-1.0, 0.0, 0.0));
        assert_eq!(orient.left(), Dir3::new(0.0, 1.0, 0.0));
        assert_eq!(orient.down(), Dir3::new(0.0, 0.0, -1.0));
    }

    /// Check that this orient struct can correctly create rays in different directions.
    #[test]
    fn ray_test() {
        let ray = Ray::new(Point3::new(0., 0., 0.), Dir3::new(1., 0., 0.));
        let orient = Orient::new(ray);

        // Check that it is in the correct position.
        assert_eq!(orient.pos(), &Point3::new(0.0, 0.0, 0.0));

        // Make sure that each of the orientations are pointing in the correct direction.
        assert_eq!(orient.forward(), &Dir3::new(1.0, 0.0, 0.0));
        assert_eq!(orient.right(), &Dir3::new(0.0, -1.0, 0.0));
        assert_eq!(orient.up(), &Dir3::new(0.0, 0.0, 1.0));

        // Check that the rays are pointing in the correct direction.
        assert_eq!(orient.forward_ray(), Ray::new(Point3::new(0., 0., 0.), Dir3::new(1., 0., 0.)));
        assert_eq!(orient.backward_ray(), Ray::new(Point3::new(0., 0., 0.), Dir3::new(-1., 0., 0.)));
        assert_eq!(orient.right_ray(), Ray::new(Point3::new(0., 0., 0.), Dir3::new(0., -1., 0.)));
        assert_eq!(orient.left_ray(), Ray::new(Point3::new(0., 0., 0.), Dir3::new(0., 1., 0.)));
        assert_eq!(orient.up_ray(), Ray::new(Point3::new(0., 0., 0.), Dir3::new(0., 0., 1.)));
        assert_eq!(orient.down_ray(), Ray::new(Point3::new(0., 0., 0.), Dir3::new(0., 0., -1.)));
    }

    /// Check that we can correctly handle the contructor case where the ray is facing along the z-axis. 
    /// In this case, we should take the up direction as being the x-axis. 
    #[test]
    fn ray_along_z_test() {
        let ray = Ray::new(Point3::new(0., 0., 0.), Dir3::new(0., 0., 1.));
        let orient = Orient::new(ray);

        // Check that it is in the correct position.
        assert_eq!(orient.pos(), &Point3::new(0.0, 0.0, 0.0));

        // Make sure that each of the orientations are pointing in the correct direction.
        assert_eq!(orient.forward(), &Dir3::new(0.0, 0.0, 1.0));
        assert_eq!(orient.right(), &Dir3::new(0.0, 1.0, 0.0));
        assert_eq!(orient.up(), &Dir3::new(1.0, 0.0, 0.0));
    }
}
