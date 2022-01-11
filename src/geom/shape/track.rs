//! Track following path enumeration.

use crate::math::{Dir3, Point3, Rot3, Vec3};
use arctk_attr::file;
use std::f64::consts::PI;

/// Line track.
#[file]
pub enum Track {
    /// Static point (pos).
    Static(Point3),
    /// Line tracking (start, end).
    Line(Point3, Point3),
    /// Circular orbit (start, center, axis).
    Circle(Point3, Vec3, Dir3),
}

impl Track {
    /// Sample the nth location.
    #[inline]
    #[must_use]
    pub fn sample(&self, n: i32, max: i32) -> Point3 {
        debug_assert!(n < max);
        debug_assert!(max > 0);

        let f = f64::from(n) / f64::from(max - 1);

        match *self {
            Self::Static(ref p) => *p,
            Self::Line(ref start, ref end) => {
                let dx = (end - start) * f;
                *start + dx
            }
            Self::Circle(ref start, ref center, ref axis) => {
                let rot = Rot3::from_axis_angle(&Vec3::from(*axis), f * 2.0 * PI);
                rot.transform_point(&(start - center)) + center
            }
        }
    }
}
