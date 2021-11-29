//! Ray implementation.

use crate::{
    access,
    math::{Dir3, Point3, Rot3, Vec3},
};

/// Ray structure.
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
