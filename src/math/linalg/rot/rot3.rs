use crate::math::{Dir3, Vec3};
use nalgebra::{Rotation3, Unit};
use serde_derive::{Deserialize, Serialize};
use std::ops::Mul;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Three-dimensional rotation.
pub struct Rot3 {
    /// Internal data.
    data: Rotation3<f64>,
}

impl Rot3 {
    pub fn new(axisangle: Vec3) -> Self {
        Rot3 {
            data: Rotation3::new(axisangle.data()),
        }
    }

    pub fn from_axis_angle(axis: &Vec3, angle: f64) -> Self {
        Rot3 {
            data: Rotation3::from_axis_angle(&Unit::new_normalize(axis.data()), angle),
        }
    }
}

impl From<Rotation3<f64>> for Rot3 {
    #[inline]
    #[must_use]
    fn from(r: Rotation3<f64>) -> Self {
        Self { data: r }
    }
}

impl Mul<Rot3> for Rot3 {
    type Output = Rot3;

    #[inline]
    #[must_use]
    fn mul(self, rhs: Rot3) -> Self {
        Self {
            data: self.data * rhs.data,
        }
    }
}

impl Mul<Dir3> for Rot3 {
    type Output = Dir3;

    #[inline]
    #[must_use]
    fn mul(self, rhs: Dir3) -> Self::Output {
        Self::Output::from(self.data * rhs.data())
    }
}

impl Mul<Vec3> for Rot3 {
    type Output = Vec3;

    #[inline]
    #[must_use]
    fn mul(self, rhs: Vec3) -> Self::Output {
        Self::Output::from(self.data * rhs.data())
    }
}
