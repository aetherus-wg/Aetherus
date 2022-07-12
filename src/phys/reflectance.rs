use crate::{
    core::Real,
    fmt_report,
    geom::{Hit, Ray, ray},
    sim::Attribute,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{f64::consts::PI, fmt::Display};

#[derive(Deserialize, Serialize, Clone, Debug)]
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
    /// A composite reflectance model combines a combination of diffuse and specular reflectance.
    /// The ratio between specular and diffuse reflection is determined by `specular_diffuse_ratio`. 
    Composite {
        diffuse_albedo: Real,
        specular_albedo: Real,
        specular_diffuse_ratio: Real,
    },
}

impl Reflectance {
    /// Produces a new Lambertian reflectance instance.
    pub fn new_lambertian(albedo: Real) -> Self {
        debug_assert!(albedo <= 1.0 && albedo >= 0.0);

        Self::Lambertian { albedo }
    }

    /// Produces a new Specular reflectance instance. 
    pub fn new_specular(albedo: Real) -> Self {
        debug_assert!(albedo <= 1.0 && albedo >= 0.0);

        Self::Specular { albedo }
    }

    /// Prodduces a new Reflectance instance that is a composite between diffuse and specular reflection.
    pub fn new_composite(diffuse_albedo: Real, specular_albedo: Real, specular_diffuse_ratio: Real) -> Self {
        debug_assert!(diffuse_albedo <= 1.0 && diffuse_albedo >= 0.0);
        debug_assert!(specular_albedo <= 1.0 && specular_albedo >= 0.0);
        debug_assert!(specular_diffuse_ratio <= 1.0 && specular_diffuse_ratio >= 0.0);

        Self::Composite { diffuse_albedo, specular_albedo, specular_diffuse_ratio }
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
            },
            Self:: Specular { ref albedo } => {
                let should_reflect = rng.gen_range(0.0..1.0) < *albedo;

                if should_reflect {
                    let p = *incident_ray.dir() + *hit.side().norm();
                    let reflected_ray = Ray::new(incident_ray.pos().clone(), *hit.side().norm() + p);
                    Some(reflected_ray)
                } else {
                    None
                }
            },
            Self::Composite { ref diffuse_albedo, ref specular_albedo, ref specular_diffuse_ratio } => {
                let is_specular = rng.gen_range(0.0..1.0) < *specular_diffuse_ratio;

                if is_specular {
                    Self::new_specular(*specular_albedo).reflect(rng, incident_ray, hit)
                } else {
                    Self::new_lambertian(*diffuse_albedo).reflect(rng, incident_ray, hit)
                }
            }
            _ => unimplemented!(),
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
            Self::Composite {
                ref diffuse_albedo,
                ref specular_albedo,
                ref specular_diffuse_ratio,
            } => {
                writeln!(fmt, "Phong: ")?;
                fmt_report!(fmt, diffuse_albedo, "diffuse albedo");
                fmt_report!(fmt, specular_albedo, "specular albedo");
                fmt_report!(fmt, specular_diffuse_ratio, "specular-to-diffuse ratio");
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Reflectance;
    use std::f64::consts::PI;
    use assert_approx_eq::assert_approx_eq;
    use statrs::statistics::Statistics;
    use crate::{
        core::Real,
        geom::{Hit, Ray, Side},
        math::{Dir3, Dir2, Point3},
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
        let norm = Dir3::new(0.0, 0.0, 1.0);
        let surf_vec = Dir2::new(1.0, 1.0);
        let reflect = Reflectance::new_lambertian(1.0);
        let attrib = Attribute::Reflector(reflect.clone());
        let hit = Hit::new(&attrib, 1.0, Side::Outside(norm));

        let mut phi_hist = Histogram::new(0.0, PI / 2.0, 90);
        // We only initialise between 0
        let mut theta_hist = Histogram::new(0.0, PI, 18);

        let mut n_killed = 0;
        let n_phot: usize = 100_000;
        let mut theta_dot_neg: usize = 0;
        for _ in 0..n_phot {
            match reflect.reflect(&mut rng, &incoming_ray, &hit) {
                Some(ray) => {
                    // Check that the outgoing ray is within the same hemisphere as the surface normal.
                    // In the case of Lambertian scattering, this is a requirement.
                    // The easy check for this is to check that norm · ray is positive.
                    assert!(ray.dir().dot(&norm) > 0.0);

                    // Sample the angle created by the ray from the normal.
                    phi_hist.collect(ray.dir().dot(&norm).acos()); 
                    
                    // Now sample the theta angle. 
                    let theta_dot = Dir2::new(ray.dir().x(), ray.dir().y()).dot(&surf_vec);
                    theta_hist.collect(theta_dot.acos());
                    // Just want to check that we are getting a uniform 360 degree coverage.
                    // The dog product will only resolve to 0 -> pi radians. 
                    if theta_dot < 0.0 { theta_dot_neg += 1; }
                }
                None => n_killed += 1,
            }
        }

        // Output for distributions. Uncomment to manually test / debug. 
        // phi_hist.save_data(std::path::Path::new("lambert_check.dat")).unwrap();
        // theta_hist.save_data(std::path::Path::new("lambert_check_theta.dat")).unwrap();

        // As the albedo is 1.0, there should be none killed.
        assert_eq!(n_killed, 0);

        // Check that the phi distribution conforms to a cos(theta) fall off with angle.  
        let norm_fac = phi_hist.iter().map(|(_b, c)| c).take(3).mean();
        for (bin, count) in phi_hist.iter() {
            // Assuming a generous threshold due to the relatively low number of photons and the nature of random draws.
            // I don't want to be triggering off false positives left, right and centre.  
            assert_approx_eq!(bin.cos(), count / norm_fac as Real, 0.1);
        }

        // Now check that the theta distribution is uniform. 
        let theta_mean = theta_hist.iter().map(|(_b, c)| c).mean();
        for (_bin, count) in theta_hist.iter() {
            // Check that we are uniform to within
            assert_approx_eq!(count, theta_mean, n_phot as Real * 0.01);
        }

        // Checkt hat we get roughly 50% of the dot products uniform, indicating
        // theta coverage across both semi-circules. 
        assert_approx_eq!(theta_dot_neg as Real, n_phot as Real * 0.5, n_phot as Real * 0.01);

    }

    #[test]
    fn test_lambertian_reflectance_semi_reflective() {
        // Create an incoming ray.
        let incoming_ray = Ray::new(Point3::new(1., 1., 0.0), Dir3::new(-1.0, -1.0, 0.0));
        let mut rng = rand::thread_rng();

        // Simulate a hit on a surface.
        let norm = Dir3::new(0.0, 0.0, 1.0);
        let surf_vec = Dir2::new(1.0, 1.0);
        let reflect = Reflectance::new_lambertian(0.5);
        let attrib = Attribute::Reflector(reflect.clone());
        let hit = Hit::new(&attrib, 1.0, Side::Outside(norm));

        // Prepare bins for capturing statistics. 
        let mut phi_hist = Histogram::new(0.0, PI / 2.0, 90);
        let mut theta_hist = Histogram::new(0.0, PI, 18);

        let mut n_killed = 0;
        let n_phot = 100_000;
        let mut theta_dot_neg: usize = 0;
        for _ in 0..n_phot {
            match reflect.reflect(&mut rng, &incoming_ray, &hit) {
                Some(ray) => {
                    // Check that the outgoing ray is within the same hemisphere as the surface normal.
                    // In the case of Lambertian scattering, this is a requirement.
                    // The easy check for this is to check that norm · ray is positive.
                    assert!(ray.dir().dot(&norm) > 0.0);

                    // Sample the angle created by the ray from the normal.
                    phi_hist.collect(ray.dir().dot(&norm).acos());
                    
                    // Now sample the theta angle. 
                    let theta_dot = Dir2::new(ray.dir().x(), ray.dir().y()).dot(&surf_vec);
                    theta_hist.collect(theta_dot.acos());
                    // Just want to check that we are getting a uniform 360 degree coverage.
                    // The dog product will only resolve to 0 -> pi radians. 
                    if theta_dot < 0.0 { theta_dot_neg += 1; }
                }
                None => n_killed += 1,
            }
        }

        // Output for distributions. Uncomment to manually test / debug. 
        // phi_hist.save_data(std::path::Path::new("lambert_check.dat")).unwrap();
        // theta_hist.save_data(std::path::Path::new("lambert_check_theta.dat")).unwrap();

        // As the albedo is 0.5, we expect roughly half of the photons to get killed.
        assert_approx_eq!(n_killed as Real, n_phot as Real * 0.5, n_phot as Real * 0.01);

        // Check that the phi distribution conforms to a cos(theta) fall off with angle.  
        let norm_fac = phi_hist.iter().map(|(_b, c)| c).take(3).mean();
        for (bin, count) in phi_hist.iter() {
            // Assuming a generous threshold due to the relatively low number of photons and the nature of random draws.
            // I don't want to be triggering off false positives left, right and centre.  
            assert_approx_eq!(bin.cos(), count / norm_fac as Real, 0.1);
        }

        // Now check that the theta distribution is uniform. 
        let theta_mean = theta_hist.iter().map(|(_b, c)| c).mean();
        for (_bin, count) in theta_hist.iter() {
            // Check that we are uniform to within
            assert_approx_eq!(count, theta_mean, (n_phot - n_killed) as Real * 0.01);
        }

        // Checkt hat we get roughly 50% of the dot products uniform, indicating
        // theta coverage across both semi-circules. 
        assert_approx_eq!(theta_dot_neg as Real, (n_phot - n_killed) as Real * 0.5, (n_phot - n_killed) as Real * 0.01);
    }
}
