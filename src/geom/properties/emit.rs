//! Emit trait.

use crate::{
    geom::Ray,
    math::{Dir3, Point3},
};
use rand::Rng;
use std::f64::consts::PI;

/// Emit trait implementation.
/// Types implementing this trait can cast Rays.
pub trait Emit {
    /// Cast a new ray.
    fn cast<R: Rng>(&self, rng: &mut R) -> Ray;
}

/// Provides a concrete implementation of ray casting for a Point3.
/// As this type is an alias, it makes the most sense to include it here. 
/// As we want to decouple the low-level linear algebra types as much as possible from
/// higher level ray-tracing code. 
impl Emit for Point3 {
    #[inline]
    #[must_use]
    fn cast<R: Rng>(&self, rng: &mut R) -> Ray {
        let theta = rng.gen_range(0.0..(2.0 * PI));
        let z = rng.gen_range(-1.0..1.0);

        Ray::new(
            *self,
            Dir3::new(
                (1.0_f64 - (z * z)).sqrt() * theta.cos(),
                (1.0_f64 - (z * z)).sqrt() * theta.sin(),
                z,
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use assert_approx_eq::assert_approx_eq;
    use super::Emit;
    use crate::{
        data::Average,
        math::Point3,
    };

    /// As the default mode of emission is isotropic, I will test this using the
    /// Point3 impl for this above. 
    #[test]
    fn point_emission_test() {
        let p = Point3::new(0.0, 0.0, 0.0);
        let mut rng = rand::thread_rng();

        let mut x_ave = Average::new();
        let mut y_ave = Average::new();
        let mut z_ave = Average::new();
        for _ in 0..10_000 {
            let ray = p.cast(&mut rng);
            x_ave += ray.dir().x();
            y_ave += ray.dir().y();
            z_ave += ray.dir().z();
        }

        // Check that we are retrieving the average of the uniform tophat correctly. 
        // Given the number of points, I would expect to get to about the 2 per cent level. 
        assert_approx_eq!(x_ave.ave(), 0.0, 0.025);
        assert_approx_eq!(y_ave.ave(), 0.0, 0.025);
        assert_approx_eq!(z_ave.ave(), 0.0, 0.025);
    }
}