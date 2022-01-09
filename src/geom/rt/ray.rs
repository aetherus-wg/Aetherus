//! Ray implementation.

use crate::{
    access,
    math::{Dir3, Point3, Rot3, Vec3},
};

/// Ray structure.
/// 
/// This is the type at the core of our ray tracing / hit scan implementation.
/// This is also the type at the core of our photon implementation. 
#[derive(Clone)]
pub struct Ray {
    /// Ray origin.
    pos: Point3,
    /// Ray direction.
    dir: Dir3,
}

impl Ray {
    access!(pos, pos_mut: Point3);
    access!(dir, dir_mut: Dir3);

    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(pos: Point3, mut dir: Dir3) -> Self {
        dir.renormalize();
        Self { pos, dir }
    }

    /// Destruct self into components.
    #[inline]
    #[must_use]
    pub const fn destruct(self) -> (Point3, Dir3) {
        (self.pos, self.dir)
    }

    /// Move along the direction of travel a given distance.
    #[inline]
    pub fn travel(&mut self, dist: f64) {
        debug_assert!(dist > 0.0);

        self.pos += self.dir * dist;
    }

    /// Rotate the photon with a given pitch and subsequent roll manoeuvre.
    #[inline]
    pub fn rotate(&mut self, pitch: f64, roll: f64) {
        let arbitrary_axis = if (1.0 - self.dir.z().abs()) >= 1.0e-1 {
            Vec3::z_axis()
        } else {
            Vec3::y_axis()
        };

        let pitch_axis = self.dir.cross(&arbitrary_axis.into());
        let pitch_rot = Rot3::from_axis_angle(&Vec3::from(pitch_axis), pitch);
        let roll_rot = Rot3::from_axis_angle(&Vec3::from(self.dir), roll);

        self.dir = roll_rot * pitch_rot * self.dir;
        self.dir.renormalize();
    }
}

#[cfg(test)]
mod tests {
    use std::f64;
    use crate::math::{Point3, Dir3};
    use super::Ray;
    use assert_approx_eq::assert_approx_eq;

    /// Check that the creation and accessing code is working correctly. 
    #[test]
    fn init_and_access_test() {
        let ray = Ray::new(Point3::new(0.0, 0.0, 0.0), Dir3::new(1.0, 1.0, 1.0));
        // Check that we get the correct
        assert_eq!(ray.pos(), &Point3::new(0.0, 0.0, 0.0));
        assert_eq!(ray.dir(), &Dir3::new(1.0, 1.0, 1.0));
    }

    /// Check that arrays destruct correctly
    #[test]
    fn ray_destruct_test() {
        let mut ray = Ray::new(Point3::new(0.0, 1.0, 0.0), Dir3::new(1.0, 0.0, 1.0));
        ray.travel(1.0);
        let (pos, dir) = ray.destruct();

        // Check each of the components of the position vector. 
        assert_approx_eq!(pos.x(), 1.0 / 2.0_f64.sqrt(), 1.0E-8);
        assert_approx_eq!(pos.y(), 1.0, 1.0E-6);
        assert_approx_eq!(pos.z(), 1.0 / 2.0_f64.sqrt(), 1.0E-8);

        // Check each of the components of the direction unit vector. 
        assert_approx_eq!(dir.x(), 1.0 / 2.0_f64.sqrt(), 1.0E-8);
        assert_approx_eq!(dir.y(), 0.0, 1.0E-6);
        assert_approx_eq!(dir.z(), 1.0 / 2.0_f64.sqrt(), 1.0E-8);
    }

    /// Check that our rays are travelling correctly. 
    #[test]
    fn travel_test() {
        let mut ray = Ray::new(Point3::new(0.0, 0.0, 0.0), Dir3::new(1.0, 0.5, 0.1));
        let norm = (1.0_f64.powf(2.0) + 0.5_f64.powf(2.0) + 0.1_f64.powf(2.0)).sqrt();
        let dist = 5.0;
        ray.travel(5.0);
        assert_approx_eq!(ray.pos().x(), (1.0 / norm) * dist, 0.001);
        assert_approx_eq!(ray.pos().y(), (0.5 / norm) * dist, 0.001);
        assert_approx_eq!(ray.pos().z(), (0.1 / norm) * dist, 0.001);
    }

    /// Check that we can correctly rotate the ray. 
    #[test]
    fn ray_rotate_test() {
        let mut ray = Ray::new(Point3::new(0.0, 0.0, 0.0), Dir3::new(0.0, 0.5, 0.0));
        ray.rotate(f64::consts::PI, 0.0);
        
        assert_approx_eq!(ray.dir().x(), 0.0);
        assert_approx_eq!(ray.dir().y(), -1.0);
        assert_approx_eq!(ray.dir().z(), 0.0);

        // Now do a roll maneuver. 
        ray.rotate(-f64::consts::PI / 2.0, -f64::consts::PI);

        assert_approx_eq!(ray.dir().x(), 0.0);
        assert_approx_eq!(ray.dir().y(), 0.0);
        assert_approx_eq!(ray.dir().z(), 1.0);
    }
}