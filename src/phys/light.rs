//! Light surface structure.

use crate::{
    access, clone, fmt_report,
    geom::Emitter,
    math::Probability,
    phys::{Material, Photon},
};
use rand::Rng;
use std::fmt::{Display, Error, Formatter};

/// Photon emission structure.
#[derive(Clone)]
pub struct Light<'a> {
    /// Power [J/s].
    power: f64,
    /// Emitter.
    emitter: Emitter,
    /// Emission spectrum.
    spec: Probability,
    /// Emitting material.
    mat: &'a Material,
}

impl<'a> Light<'a> {
    clone!(power: f64);
    access!(spec: Probability);
    access!(mat: Material);

    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(power: f64, emitter: Emitter, spec: Probability, mat: &'a Material) -> Self {
        debug_assert!(power > 0.0);

        Self {
            power,
            emitter,
            spec,
            mat,
        }
    }

    /// Emit a new photon.
    #[inline]
    #[must_use]
    pub fn emit<R: Rng>(&self, mut rng: &mut R, power: f64) -> Photon {
        debug_assert!(power > 0.0);

        let ray = self.emitter.emit(&mut rng);
        let wavelength = self.spec.sample(&mut rng);

        Photon::new(ray, wavelength, power)
    }
}

impl<'a> Display for Light<'a> {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        writeln!(fmt, "...")?;
        fmt_report!(fmt, self.power, "power (J/s)");
        fmt_report!(fmt, self.emitter, "emitter");
        fmt_report!(fmt, self.spec, "emission spectrum");
        fmt_report!(fmt, self.mat, "emission material");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand;
    use crate::{
        math::{Formula, Probability, Point3, Dir3},
        phys::{Material},
        geom::{Emitter, Ray, Mesh, SmoothTriangle, Triangle},
    };
    use assert_approx_eq::assert_approx_eq;

    fn get_air_material() -> Material {
        Material::new(
            Formula::Constant { c: 1.0 }, 
            Formula::Constant { c: 1.0e-6 }, 
            None, 
            None, 
            Formula::Constant { c: 0.1 }
        )
    }

    #[test]
    fn test_beam_light() {
        let mut rng = rand::thread_rng();
        let ray = Ray::new(Point3::new(0.0, 0.0, 0.0), Dir3::new(1.0, 0.0, 0.0));
        let emitter = Emitter::new_beam(ray.clone());
        let mat = get_air_material();
        let light = Light::new(1.0, emitter, Probability::new_point(1.0), &mat);
        
        // Now emit a photon and check we get the correct values. 
        let photon = light.emit(&mut rng, 1.0);
        assert_eq!(photon.power(), 1.0);
        assert_eq!(photon.wavelength(), 1.0);
        assert_eq!(photon.ray(), &ray);
    }

    #[test]
    fn test_points_light() {
        let mut rng = rand::thread_rng();
        let points = vec![Point3::new(1.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.0), Point3::new(0.0, 0.0, 1.0)];
        let emitter = Emitter::new_points(points.clone());
        let mat = get_air_material();
        let light = Light::new(1.0, emitter, Probability::new_point(1.0), &mat);
        
        // Now emit a photon and check we get the correct values. 
        let photon = light.emit(&mut rng, 1.0);
        assert_eq!(photon.power(), 1.0);
        assert_eq!(photon.wavelength(), 1.0);
        assert!(points.contains(&photon.ray().pos()));
    }

    #[test]
    fn test_weighted_points() {
        let mut rng = rand::thread_rng();
        let points = vec![Point3::new(1.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.0), Point3::new(0.0, 0.0, 1.0)];
        let weights = [1.0, 2.0, 3.0];
        let emitter = Emitter::new_weighted_points(points.clone(), &weights);
        let mat = get_air_material();
        let light = Light::new(1.0, emitter, Probability::new_point(1.0), &mat);
        
        // Now emit a number of photons and check that the weights are correct. 
        let mut freqs = vec![0, 0, 0];
        let n_samples = 100_000;
        for _ in 0..n_samples {
            let photon = light.emit(&mut rng, 1.0);
            assert_eq!(photon.power(), 1.0);
            assert_eq!(photon.wavelength(), 1.0);
            assert!(points.contains(&photon.ray().pos()));

            let index = points.iter().position(|p| p == photon.ray().pos()).unwrap();
            freqs[index] += 1;
        }

        // Now check that the weights are correct.
        assert_approx_eq!(freqs[0] as f64/ freqs[1] as f64, 0.5, 0.01);
        assert_approx_eq!(freqs[0] as f64/ freqs[2] as f64, 1.0/3.0, 0.01);
    }

    #[test]
    fn test_surface_light() {
        let mut rng = rand::thread_rng();

        // Make a single upward facing triangle to emit from. 
        let norm = Dir3::new(0.0, 0.0, 1.0);
        let triangles = vec![ SmoothTriangle::new(
            Triangle::new([
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(1.0, 0.0, 0.0),
                Point3::new(0.0, 1.0, 0.0),
        ]),
            [Dir3::new(0.0, 0.0, 1.0), Dir3::new(0.0, 0.0, 1.0), Dir3::new(0.0, 0.0, 1.0)],
        )];
        let mesh = Mesh::new(triangles);

        let emitter = Emitter::new_surface(mesh);
        let mat = get_air_material();
        let light = Light::new(1.0, emitter, Probability::new_point(1.0), &mat);
        
        // Now emit a photon and check we get the correct values. 
        let photon = light.emit(&mut rng, 1.0);
        assert_eq!(photon.power(), 1.0);
        assert_eq!(photon.wavelength(), 1.0);
        assert_eq!(photon.ray().dir(), &norm);
        assert_approx_eq!(photon.ray().pos().z(), 0.0, 1.0e-6);
    }
}