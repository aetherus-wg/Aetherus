use crate::{
    core::Real,
    fmt_report,
    geom::{Hit, Ray},
    sim::Attribute,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{f64::consts::PI, fmt::Display};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum Reflectance {
    /// Lambertian Reflectance
    ///
    /// Provides a purely diffuse reflectance, and reflects evenly in the hemisphere
    /// around the normal vector, irrespective of the direction of the incident
    /// light ray.
    /// The albdeo determines which portion of the incoming photons are reflected or killed.
    /// An albedo of 1.0 will reflect all photons, and 0.0 will kill all photons.
    Lambertian { albedo: Real },
    /// Specular Reflectance
    ///
    /// Provides a purely specular reflectance, where the angle of the reflected
    /// photon from the normal vector is the same as the incoming ray.
    /// The albdeo determines which portion of the incoming photons are reflected or killed.
    /// An albedo of 1.0 will reflect all photons, and 0.0 will kill all photons.
    Specular { albedo: Real },
    /// Composition Reflectance Model - Specular + Diffuse
    ///
    /// A composite reflectance model combines a combination of diffuse and specular reflectance.
    /// The ratio between diffuse and soecular reflection is determined by `specular_diffuse_ratio`,
    /// with 1.0 corresponding to pure diffuse and 0.0 corresponding to pure specular.
    /// The `diffuse_albedo` and `specular_albedo` are directly fed through to the
    /// albedos of their respective models and have their usual meanings.
    Composite {
        diffuse_albedo: Real,
        specular_albedo: Real,
        diffuse_specular_ratio: Real,
    },
}

impl Reflectance {
    /// Produces a new Lambertian reflectance instance.
    /// This returns a purely diffuse reflection.
    /// In this case photons are randomly distributed in the hemisphere in which
    /// the normal to the surface lies.
    /// The albdeo determines which portion of the incoming photons are reflected or killed.
    /// An albedo of 1.0 will reflect all photons, and 0.0 will kill all photons.
    pub fn new_lambertian(albedo: Real) -> Self {
        debug_assert!(albedo <= 1.0 && albedo >= 0.0);

        Self::Lambertian { albedo }
    }

    /// Produces a new Specular reflectance instance.
    /// This returns a purely specular reflection. In this case the incoming photons
    /// are reflected like they would be from a mirror; at the same angle to the normal vector of the surface.
    /// The albdeo determines which portion of the incoming photons are reflected or killed.
    /// An albedo of 1.0 will reflect all photons, and 0.0 will kill all photons.
    pub fn new_specular(albedo: Real) -> Self {
        debug_assert!(albedo <= 1.0 && albedo >= 0.0);

        Self::Specular { albedo }
    }

    /// Prodduces a new Reflectance instance that is a composite between diffuse and specular reflection.
    /// This is a combination of diffuse (Lambertian) and specular reflection, with the ratio between them
    /// determined by the `specular_diffuse_ratio`. 1.0 corresponds to pure diffuse and 0.0 corresponds to pure specular.
    pub fn new_composite(
        diffuse_albedo: Real,
        specular_albedo: Real,
        diffuse_specular_ratio: Real,
    ) -> Self {
        debug_assert!(diffuse_albedo <= 1.0 && diffuse_albedo >= 0.0);
        debug_assert!(specular_albedo <= 1.0 && specular_albedo >= 0.0);
        debug_assert!(diffuse_specular_ratio <= 1.0 && diffuse_specular_ratio >= 0.0);

        Self::Composite {
            diffuse_albedo,
            specular_albedo,
            diffuse_specular_ratio,
        }
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
                // This random draw determines if the photon should reflect, based on the value of the albedo.
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
            Self::Specular { ref albedo } => {
                // This random draw determines if the photon should reflect, based on the value of the albedo.
                let should_reflect = rng.gen_range(0.0..1.0) < *albedo;

                if should_reflect {
                    // Implementation for this heavily borrowed from: https://www.cs.uaf.edu/2006/fall/cs381/lecture/10_03_specular.html
                    let reflect = *incident_ray.dir()
                        + 2.0 * hit.side().norm().dot(&-*incident_ray.dir()) * hit.side().norm();
                    let reflected_ray = Ray::new(incident_ray.pos().clone(), reflect.into());
                    Some(reflected_ray)
                } else {
                    None
                }
            }
            Self::Composite {
                ref diffuse_albedo,
                ref specular_albedo,
                ref diffuse_specular_ratio,
            } => {
                // This random draw determines, based on the ratio, whether the reflection for the
                // current photon should be diffuse (Lambertian) or specular.
                let is_specular = rng.gen_range(0.0..1.0) > *diffuse_specular_ratio;

                // Then we just delegate handling of the reflection to the respective model.
                if is_specular {
                    Self::new_specular(*specular_albedo).reflect(rng, incident_ray, hit)
                } else {
                    Self::new_lambertian(*diffuse_albedo).reflect(rng, incident_ray, hit)
                }
            }
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
                ref diffuse_specular_ratio,
            } => {
                writeln!(fmt, "Composite: ")?;
                fmt_report!(fmt, diffuse_albedo, "diffuse albedo");
                fmt_report!(fmt, specular_albedo, "specular albedo");
                fmt_report!(fmt, diffuse_specular_ratio, "diffuse-to-specular ratio");
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Reflectance;
    use crate::{
        core::Real,
        data::Histogram,
        geom::{Hit, Ray, Side},
        math::{Dir2, Dir3, Point3},
        sim::Attribute,
    };
    use assert_approx_eq::assert_approx_eq;
    use rand;
    use statrs::statistics::Statistics;
    use std::f64::consts::PI;

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
                    // The easy check for this is to check that norm ?? ray is positive.
                    assert!(ray.dir().dot(&norm) > 0.0);

                    // Sample the angle created by the ray from the normal.
                    phi_hist.collect(ray.dir().dot(&norm).acos());

                    // Now sample the theta angle.
                    let theta_dot = Dir2::new(ray.dir().x(), ray.dir().y()).dot(&surf_vec);
                    theta_hist.collect(theta_dot.acos());
                    // Just want to check that we are getting a uniform 360 degree coverage.
                    // The dog product will only resolve to 0 -> pi radians.
                    if theta_dot < 0.0 {
                        theta_dot_neg += 1;
                    }
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
        assert_approx_eq!(
            theta_dot_neg as Real,
            n_phot as Real * 0.5,
            n_phot as Real * 0.01
        );
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
                    // The easy check for this is to check that norm ?? ray is positive.
                    assert!(ray.dir().dot(&norm) > 0.0);

                    // Sample the angle created by the ray from the normal.
                    phi_hist.collect(ray.dir().dot(&norm).acos());

                    // Now sample the theta angle.
                    let theta_dot = Dir2::new(ray.dir().x(), ray.dir().y()).dot(&surf_vec);
                    theta_hist.collect(theta_dot.acos());
                    // Just want to check that we are getting a uniform 360 degree coverage.
                    // The dog product will only resolve to 0 -> pi radians.
                    if theta_dot < 0.0 {
                        theta_dot_neg += 1;
                    }
                }
                None => n_killed += 1,
            }
        }

        // Output for distributions. Uncomment to manually test / debug.
        // phi_hist.save_data(std::path::Path::new("lambert_check.dat")).unwrap();
        // theta_hist.save_data(std::path::Path::new("lambert_check_theta.dat")).unwrap();

        // As the albedo is 0.5, we expect roughly half of the photons to get killed.
        assert_approx_eq!(
            n_killed as Real,
            n_phot as Real * 0.5,
            n_phot as Real * 0.01
        );

        // Check that the phi distribution conforms to a cos(theta) fall off with angle.
        let norm_fac = phi_hist.iter().map(|(_b, c)| c).take(3).mean();
        for (bin, count) in phi_hist.iter() {
            // Assuming a generous threshold due to the relatively low number of photons and the nature of random draws.
            // I don't want to be triggering off false positives left, right and centre.
            assert_approx_eq!(bin.cos(), count / norm_fac as Real, 0.15);
        }

        // Now check that the theta distribution is uniform.
        let theta_mean = theta_hist.iter().map(|(_b, c)| c).mean();
        for (_bin, count) in theta_hist.iter() {
            // Check that we are uniform to within
            assert_approx_eq!(count, theta_mean, (n_phot - n_killed) as Real * 0.01);
        }

        // Checkt hat we get roughly 50% of the dot products uniform, indicating
        // theta coverage across both semi-circules.
        assert_approx_eq!(
            theta_dot_neg as Real,
            (n_phot - n_killed) as Real * 0.5,
            (n_phot - n_killed) as Real * 0.01
        );
    }

    /// This is a test of the specular reflection code. This should be a lot easier
    /// than the diffuse tests. As we know the incoming ray, and normal vector,
    /// we can analytically find the reflected ray.
    #[test]
    fn test_specular_reflectance_perfect_reflector() {
        // Create an incoming ray.
        let incoming_ray = Ray::new(Point3::new(1., 0., 1.0), Dir3::new(1.0, 0.0, -1.0));
        let mut rng = rand::thread_rng();

        // Simulate a hit on a surface.
        let norm = Dir3::new(0.0, 0.0, 1.0);
        let reflect = Reflectance::new_specular(1.0);
        let attrib = Attribute::Reflector(reflect.clone());
        let hit = Hit::new(&attrib, 2.0_f64.sqrt(), Side::Outside(norm));

        // Expected output - analytically determined by working through the equations.
        let reflected_ray_test = Ray::new(Point3::new(1.0, 0.0, 1.0), Dir3::new(1.0, 0.0, 1.0));

        match reflect.reflect(&mut rng, &incoming_ray, &hit) {
            Some(ray) => {
                // Use assert_approx_eq due to numerical noise.
                assert_approx_eq!(ray.dir().dot(reflected_ray_test.dir()), 1.0);
            }
            None => assert!(false), // With a perfect reflector, we should have no killed photons.
        }
    }

    #[test]
    fn test_specular_reflectance_semi_reflective() {
        // Create an incoming ray.
        let incoming_ray = Ray::new(Point3::new(1., 0., 1.0), Dir3::new(1.0, 0.0, -1.0));
        let mut rng = rand::thread_rng();

        // Simulate a hit on a surface.
        let norm = Dir3::new(0.0, 0.0, 1.0);
        let reflect = Reflectance::new_specular(0.5);
        let attrib = Attribute::Reflector(reflect.clone());
        let hit = Hit::new(&attrib, 2.0_f64.sqrt(), Side::Outside(norm));

        // Expected output - analytically determined by working through the equations.
        let reflected_ray_test = Ray::new(Point3::new(1.0, 0.0, 1.0), Dir3::new(1.0, 0.0, 1.0));

        // Register killed photons.
        let n_photon: usize = 100_000;
        let mut n_killed_photons: usize = 0;
        for _ in 0..n_photon {
            match reflect.reflect(&mut rng, &incoming_ray, &hit) {
                Some(ray) => {
                    // Use assert_approx_eq due to numerical noise.
                    assert_approx_eq!(ray.dir().dot(reflected_ray_test.dir()), 1.0);
                }
                None => n_killed_photons += 1, // With a perfect reflector, we should have no killed photons.
            }
        }

        // Now check that the kill-rate of photons is consistent with the albedo.
        println!("{}", n_killed_photons);
        assert_approx_eq!(
            n_killed_photons as Real,
            n_photon as Real * 0.5,
            n_photon as Real * 0.01
        );
    }

    #[test]
    fn test_composite_reflectance() {
        // Create an incoming ray.
        let incoming_ray = Ray::new(Point3::new(0., 1., 1.0), Dir3::new(0.0, 1.0, -1.0));
        let mut rng = rand::thread_rng();

        // Simulate a hit on a surface.
        let norm = Dir3::new(0.0, 0.0, 1.0);
        let surf_vec = Dir2::new(1.0, 0.0);
        let reflect = Reflectance::new_composite(1.0, 1.0, 0.5);
        let attrib = Attribute::Reflector(reflect.clone());
        let hit = Hit::new(&attrib, 2.0_f64.sqrt(), Side::Outside(norm));

        // Prepare bins for capturing statistics.
        let mut phi_hist = Histogram::new(0.0, PI / 2.0, 90);
        let mut theta_hist = Histogram::new(0.0, PI, 180);

        let n_phot = 100_000;
        let mut theta_dot_neg: usize = 0;
        for _ in 0..n_phot {
            match reflect.reflect(&mut rng, &incoming_ray, &hit) {
                Some(ray) => {
                    // Check that the outgoing ray is within the same hemisphere as the surface normal.
                    // In the case of Lambertian scattering, this is a requirement.
                    // The easy check for this is to check that norm ?? ray is positive.
                    assert!(ray.dir().dot(&norm) > 0.0);

                    // Sample the angle created by the ray from the normal.
                    phi_hist.collect(ray.dir().dot(&norm).acos());

                    // Now sample the theta angle.
                    let theta_dot = Dir2::new(ray.dir().x(), ray.dir().y()).dot(&surf_vec);
                    theta_hist.collect(theta_dot.acos());
                    // Just want to check that we are getting a uniform 360 degree coverage.
                    // The dog product will only resolve to 0 -> pi radians.
                    if theta_dot < 0.0 {
                        theta_dot_neg += 1;
                    }
                }
                None => assert!(false),
            }
        }

        // Output for distributions. Uncomment to manually test / debug.
        // phi_hist.save_data(std::path::Path::new("composite_check.dat")).unwrap();
        // theta_hist.save_data(std::path::Path::new("composite_check_theta.dat")).unwrap();

        // Initialise our models for comparison.
        // The specular model is consistent between the theta and phi axes as it
        // manifests as a constant n_photon / 2 term that occurs in a single bin.
        let specular_component =
            |ibin: usize, target_bin: usize, nphot: usize, ratio: Real, albedo: Real| {
                if ibin == target_bin {
                    nphot as Real * albedo * ratio
                } else {
                    0.0
                }
            };
        // The diffuse component is more complicated as in theta it merely dilutes
        // the the entire n_photon / 2 allocation over all bins in the histogram.
        // However in the phi component there is a cos(phi) dependence, which we
        // model by borrowing the method from out lambertian reflectance tests above.
        let diff_norm_fac_phi = phi_hist.iter().map(|(_b, c)| c).take(3).mean();
        let diffuse_component_phi = |bin: Real, norm_fac: Real| bin.cos() * norm_fac;
        let diffuse_component_theta = |nbin: usize, nphot: usize, ratio: Real, albedo: Real| {
            (albedo * nphot as Real * ratio) / nbin as Real
        };

        for (ibin, (bin, count)) in phi_hist.iter().enumerate() {
            // The bins are at 1 degree increments. As we are testing relative to the x-axis, the reflection should be at 90 degrees, and hence should be in the 90th bin.
            let model_count = diffuse_component_phi(bin, diff_norm_fac_phi)
                + specular_component(ibin, 45, n_phot, 0.5, 1.0);
            // We are checking that we agree to about the 1% level.
            assert_approx_eq!(model_count, count, n_phot as Real * 0.01);
        }

        for (ibin, (_, count)) in theta_hist.iter().enumerate() {
            // The bins are at 1 degree increments. As we are testing relative to the x-axis, the reflection should be at 90 degrees, and hence should be in the 90th bin.
            let model_count = diffuse_component_theta(theta_hist.binner().bins(), n_phot, 0.5, 1.0)
                + specular_component(ibin, 90, n_phot, 0.5, 1.0);
            // We are checking that we agree to about the 1% level.
            assert_approx_eq!(model_count, count, n_phot as Real * 0.01);
        }

        // Checkt hat we get roughly 25% of the dot products uniform, indicating
        // theta coverage across both semi-circules.
        assert_approx_eq!(
            theta_dot_neg as Real,
            n_phot as Real * 0.25,
            n_phot as Real * 0.01
        );
    }
}
