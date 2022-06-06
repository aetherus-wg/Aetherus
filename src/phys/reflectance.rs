use crate::{
    core::Real,
    fmt_report,
    geom::{Hit, Ray},
    math::{Dir3, Rot3, Trans3, Trans3Builder, Vec3},
    sim::Attribute,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{f64::consts::PI, fmt::Display};

#[derive(Deserialize, Serialize, Clone)]
pub enum Reflectance {
    /// Lambertian Reflectance.
    ///
    /// Provides a purely diffuse reflectance, and reflects evenly in the hemisphere
    /// around the normal vector, irrespective of the direction of the incident
    /// light ray.
    Lambertian { albedo: Real },
    /// Specular Reflectance. (TODO)
    Specular { albedo: Real },
    /// Phong Reflectance. (TODO)
    ///
    /// A Phong reflectance model combines a combination of diffuse and specular reflectance.
    Phong {
        diffuse_albedo: Real,
        specular_albedo: Real,
    },
}

impl Reflectance {
    /// Produces a new Lambertian reflectance instance.
    pub fn new_lambertian(albedo: Real) -> Self {
        Self::Lambertian { albedo }
    }

    /// Provided an incident ray, this will reflect the raw according to the
    /// reflectance model that is used. Note that the returned ray can be an
    /// option. In the case that `None` is returned, this is indicative that the
    /// ray should not be reflected, and should be destroyed.
    #[inline]
    pub fn reflect<R: Rng>(
        &self,
        rng: &mut R,
        incident_ray: &Ray,
        hit: &Hit<Attribute>,
    ) -> Option<Ray> {
        match *self {
            Self::Lambertian { ref albedo } => {
                let should_reflect = rng.gen_range(0.0..1.0) < *albedo;

                if should_reflect {
                    let theta = rng.gen_range(0.0..2.0 * PI);
                    // We sample the phi angle using PDF = sin(theta)
                    let phi = (rng.gen_range(0.0..1.0) as Real).asin();

                    let mut reflected_ray =
                        Ray::new(incident_ray.pos().clone(), hit.side().norm().clone());
                    reflected_ray.rotate(phi, theta);
                    Some(reflected_ray)
                } else {
                    None
                }
            }
            _ => todo!(),
        }
    }
}

impl Display for Reflectance {
    #[inline]
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Lambertian { ref albedo } => {
                writeln!(fmt, "Lambertian: ")?;
                fmt_report!(fmt, albedo, "albedo");
                Ok(())
            }
            Self::Specular { ref albedo } => {
                writeln!(fmt, "Specular: ")?;
                fmt_report!(fmt, albedo, "albedo");
                Ok(())
            }
            Self::Phong {
                ref diffuse_albedo,
                ref specular_albedo,
            } => {
                writeln!(fmt, "Phong: ")?;
                fmt_report!(fmt, diffuse_albedo, "disffuse albedo");
                fmt_report!(fmt, specular_albedo, "specular albedo");
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Reflectance;
    use std::f64::consts::PI;
    use crate::{
        geom::{Hit, Ray, Side},
        math::{Dir3, Point3},
        data::Histogram,
        sim::Attribute, fs::Save,
    };
    use rand;

    #[test]
    fn test_lambertian_reflectance_perfect_reflector() {
        // Create an incoming ray.
        let incoming_ray = Ray::new(Point3::new(1., 1., 0.0), Dir3::new(-1.0, -1.0, 0.0));
        let mut rng = rand::thread_rng();

        // Simulate a hit on a surface.
        let norm = Dir3::new(1.0, 1.0, 1.0);
        let reflect = Reflectance::new_lambertian(1.0);
        let attrib = Attribute::Reflector(reflect.clone());
        let hit = Hit::new(&attrib, 1.0, Side::Outside(norm));

        let mut phi_hist = Histogram::new(0.0, PI / 2.0, 90);
        let theta_hist = Histogram::new(0.0, 2.0 * PI, 36);

        let mut n_killed = 0;
        for _ in 0..10_000 {
            match reflect.reflect(&mut rng, &incoming_ray, &hit) {
                Some(ray) => {
                    // Check that the outgoing ray is within the same hemisphere as the surface normal.
                    // In the case of Lambertian scattering, this is a requirement.
                    // The easy check for this is to check that norm · ray is positive.
                    assert!(ray.dir().dot(&norm) > 0.0);

                    // Sample the angle created by the ray from the normal.
                    phi_hist.collect(ray.dir().dot(&norm).acos());
                }
                None => n_killed += 1,
            }
        }

        // As the albedo is 1.0, there should be none killed.
        assert_eq!(n_killed, 0);

        // Check that the distribution of angles is correct.
        phi_hist.save_data(std::path::Path::new("lambert_check.dat")).unwrap();
    }

    #[test]
    fn test_lambertian_reflectance_semi_reflective() {
        // Create an incoming ray.
        let incoming_ray = Ray::new(Point3::new(1., 1., 0.0), Dir3::new(-1.0, -1.0, 0.0));
        let mut rng = rand::thread_rng();

        // Simulate a hit on a surface.
        let norm = Dir3::new(1.0, 1.0, 1.0);
        let reflect = Reflectance::new_lambertian(0.5);
        let attrib = Attribute::Reflector(reflect.clone());
        let hit = Hit::new(&attrib, 1.0, Side::Outside(norm));

        let mut n_killed = 0;
        for i in 0..10_000 {
            match reflect.reflect(&mut rng, &incoming_ray, &hit) {
                Some(ray) => {
                    // Check that the outgoing ray is within the same hemisphere as the surface normal.
                    // In the case of Lambertian scattering, this is a requirement.
                    // The easy check for this is to check that norm · ray is positive.
                    assert!(ray.dir().dot(&norm) > 0.0);
                }
                None => n_killed += 1,
            }
        }

        // As the albedo is 0.5, we expect roughly half of the photons to get killed.
        assert!(n_killed > 4900 && n_killed < 5100);
    }
}
