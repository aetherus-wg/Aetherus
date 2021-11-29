//! Three-dimensional unit vector.

use crate::{core::Real, math::Vec3, clone};
use nalgebra::{Unit, Vector3, Const};
use std::ops::{Mul, Neg, Add};
use serde_derive::{Serialize, Deserialize};

/// Normalised three dimensional real-number vector.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Dir3 {
    /// Internal data.
    data: Unit<Vector3<Real>>,
}

impl Dir3 {
    clone!(data: Unit<Vector3<Real>>);

    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(x: Real, y: Real, z: Real) -> Self {
        Self {
            data: Unit::new_normalize(Vector3::new(x, y, z)),
        }
    }

    /// Access the first component.
    #[inline]
    #[must_use]
    pub fn x(&self) -> Real {
        self.data.x
    }

    /// Access the second component.
    #[inline]
    #[must_use]
    pub fn y(&self) -> Real {
        self.data.y
    }

    /// Access the third component.
    #[inline]
    #[must_use]
    pub fn z(&self) -> Real {
        self.data.z
    }

    /// Construct a x-axis column vector.
    #[inline]
    #[must_use]
    pub fn x_axis() -> Dir3 {
        Self::new(1.0, 0.0, 0.0)
    }

    /// Construct a x-axis column vector.
    #[inline]
    #[must_use]
    pub fn y_axis() -> Dir3 {
        Self::new(0.0, 1.0, 0.0)
    }

    /// Construct a x-axis column vector.
    #[inline]
    #[must_use]
    pub fn z_axis() -> Dir3 {
        Self::new(0.0, 0.0, 1.0)
    }

    #[inline]
    #[must_use]
    pub fn cross(&self, b: &Vec3) -> Vec3 {
        Vec3::from(self.data.cross(&b.data()))
    }

    #[inline]
    #[must_use]
    pub fn cross_dir(&self, b: &Dir3) -> Vec3 {
        Vec3::from(self.data.cross(&b.data()))
    }

    #[inline]
    #[must_use]
    pub fn dot(&self, b: &Dir3) -> f64 {
        self.data.dot(&b.data)
    }

    #[inline]
    #[must_use]
    pub fn renormalize(&mut self) {
        self.data.renormalize();
    }

    #[inline]
    pub fn iter(&self) -> nalgebra::base::iter::MatrixIter<'_, Real, Const<3>, Const<1>, nalgebra::ArrayStorage<Real, 3, 1>> {
        self.data.iter()
    }
}

impl From<Vec3> for Dir3 {
    #[inline]
    #[must_use]
    fn from(d: Vec3) -> Self {
        Self { data: Unit::new_normalize(d.data()) }
    }
}

impl From<Vector3<Real>> for Dir3 {
    #[inline]
    #[must_use]
    fn from(d: Vector3<Real>) -> Self {
        Self { data: Unit::new_normalize(d) }
    }
}

impl From<Unit<Vector3<Real>>> for Dir3 {
    #[inline]
    #[must_use]
    fn from(d: Unit<Vector3<Real>>) -> Self {
        Self { data: d }
    }
}

impl Mul<f64> for Dir3 {
    type Output = Vec3;

    fn mul(self, rhs: f64) -> Vec3 {
        return Vec3::new(self.x() * rhs, self.y() * rhs, self.z() * rhs)
    }
}

impl Mul<Dir3> for f64 {
    type Output = Vec3;

    #[inline]
    #[must_use]
    fn mul(self, rhs: Dir3) -> Vec3 {
        Vec3::new(rhs.data().x * self, rhs.data().y * self, rhs.data().z * self)
    }
}

impl Mul<&Dir3> for f64 {
    type Output = Vec3;

    #[inline]
    #[must_use]
    fn mul(self, rhs: &Dir3) -> Vec3 {
        Vec3::new(rhs.data().x * self, rhs.data().y * self, rhs.data().z * self)
    }
}

impl Neg for Dir3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        return Self::new(-self.x(), -self.y(), self.z())
    }
}

impl Add<Dir3> for Dir3 {
    type Output = Dir3;

    fn add(self, rhs: Dir3) -> Self::Output {
        Dir3::new( self.data.x + rhs.data.x, self.data.y + rhs.data.y, self.data.z + rhs.data.z )
    }
}

impl PartialOrd for Dir3 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.data.partial_cmp(&other.data)
    }
}

impl PartialEq for Dir3 {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}