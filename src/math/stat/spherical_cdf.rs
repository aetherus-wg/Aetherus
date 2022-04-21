use crate::{
    access, core::Real, math::{
        rng::Probability,
        linalg::Dir2
    }, geom::ray
};
use lidrs::photweb::{PhotometricWeb, PlaneWidth};
use rand::Rng;
use ndarray::Array1;
use statrs::statistics::Statistics;
use std::{
    f64::consts::PI,
    io::Write,
};
use cubic_splines;

/// This is the target number of polar angles that we are aiming for for the spherical CDF.
/// If more than this, we will not interpolate, however, if less we will interpolate data points
/// to ensure that we can sample the sin(theta) area term well enough. 
const TARGET_NANGLES: usize = 360;

#[derive(Debug)]
pub struct SphericalCdfPlane {
    /// The central azimurhal angle of the plane.
    azimuth_angle: Real,
    /// The angular diameter of the plane in the azimuthal axis.
    delta_aziumuth: PlaneWidth,
    cdf: Probability,
}

impl SphericalCdfPlane {
    access!(azimuth_angle, azimuth_angle_mut: Real);
    access!(delta_aziumuth, delta_aziumuth_mut: PlaneWidth);
    access!(cdf, cdf_mut: Probability);

    pub fn new() -> Self {
        Self {
            azimuth_angle: 0.,
            delta_aziumuth: PlaneWidth::new(),
            cdf: Probability::new_point(0.),
        }
    }

    /// Checks to see if the azimuthal angle is contained within the curren plane.
    /// This will return true if the angle is in the plane, else it will return false.
    pub fn azimuthal_angle_in_plane(&self, azimuthal_angle: Real) -> bool {

        let plane_dir = Dir2::new(self.azimuth_angle.sin(), self.azimuth_angle.cos());
        let ray_dir = Dir2::new(azimuthal_angle.sin(), azimuthal_angle.cos());
        let dot_prod = plane_dir.dot(&ray_dir);
        let dtheta = dot_prod.acos();

        let half_dazimuth = if (azimuthal_angle - self.azimuth_angle).sin() < 0.0 {
            self.delta_aziumuth.lower()
        } else {
            self.delta_aziumuth.upper()
        };

        dtheta <= half_dazimuth
    }

    /// Sample the CDF of this plane to return an angle consistent with this CDF.
    pub fn sample<R: Rng>(&self, rng: &mut R) -> Real {
        self.cdf.sample(rng)
    }
}

#[derive(Debug)]
/// The spherical CDF object.
pub struct SphericalCdf {
    planes: Vec<SphericalCdfPlane>,
    azimuth_cdf: Probability,
}

impl SphericalCdf {
    access!(planes, planes_mut: Vec<SphericalCdfPlane>);
    access!(azimuth_cdf, azimuth_cdf_mut: Probability);

    /// Returns a new default spherical cumulative distribution function.
    pub fn new() -> Self {
        Self {
            planes: vec![],
            azimuth_cdf: Probability::new_point(0.0),
        }
    }

    /// Returns true if the distribution is spherically symmetric - in this case there will only be one plane.
    pub fn is_spherically_symmetric(&self) -> bool {
        self.planes.iter().count() == 1
    }

    /// Samples the CDF and returns a tuple containing the azimuthal and polar angles in radians.
    /// These angles are randomly chosen based on the underlying CDF.
    pub fn sample<R: Rng>(&self, rng: &mut R) -> (Real, Real) {
        // First, draw the azimuthal angle from the azimuthal CDF and apply the offset to map back into an appropriate system.
        let azim_draw = self.azimuth_cdf.sample(rng);

        // Now find the plane for which this azimuthal angle corresponds, so that we can sampe polar angle.
        let iplane = self
            .planes
            .iter()
            .position(|pl| pl.azimuthal_angle_in_plane(azim_draw))
            .unwrap();
        let polar_draw = self.planes[iplane].sample(rng);

        (azim_draw, polar_draw)
    }
}

impl From<PhotometricWeb> for SphericalCdf {
    /// The main function that does the conversion from `lidrs::photweb::PhotometricWeb` to a spherical
    /// CDF that Aetherus can understand.
    fn from(photweb: PhotometricWeb) -> SphericalCdf {
        let mut cdf = SphericalCdf::new();

        let mut azim_angles: Vec<Real> = Vec::with_capacity(photweb.n_planes() + 1);
        let mut azim_probs: Vec<Real> = Vec::with_capacity(photweb.n_planes() + 1);

        // Calculate the azimuth angles.
        let total_intens = photweb.total_intensity() + photweb.planes()[0].integrate_intensity();

        // Iterate through the planes in the photometric web and convert them to CDFs for each plane.
        let cdf_planes = photweb
            .planes()
            .iter()
            .enumerate()
            .map(|(iplane, plane)| {
                // First calculate the normalised probabilities for reach of the angles - this is our PDF.
                let plane_intensity = plane.integrate_intensity();
                
                // Iterate through the surfaces in the current plane to get the spline points for the CDF.
                debug_assert!(plane.n_samples() > 0);
                let mut probs = Vec::with_capacity(plane.n_samples());
                let mut angles = Vec::with_capacity(plane.n_samples());

                for (ipts, intens) in plane.intensities().iter().enumerate() {
                    probs.push((intens * plane.delta_angle(ipts) * plane.width().total()) / plane_intensity);
                    angles.push(plane.angles()[ipts]);
                }

                // If the number of angles in the CDF is too small, we should upsample using an interpolation. 
                if plane.n_samples() < TARGET_NANGLES {

                    let keys: Vec<(Real, Real)> = angles.iter()
                        .zip(probs)
                        .map(|(ang, prob)| {
                            //Key::new(*ang, prob, Interpolation::Linear)
                            (*ang, prob)
                        })
                        .collect();
                    //let prob_spline = Spline::from_vec(keys);
                    let prob_spline = cubic_splines::Spline::new(keys.clone(), cubic_splines::BoundaryCondition::Natural);

                    // Clear the probs and angles vectors.
                    probs = Vec::with_capacity(TARGET_NANGLES + 1);
                    angles = Vec::with_capacity(TARGET_NANGLES + 1);

                    // Interpolate this term.
                    let min_angle = plane.angles().min();
                    let max_angle = plane.angles().max();
                    let dtheta = (max_angle - min_angle) / (TARGET_NANGLES as Real);
                    for iang in 0..TARGET_NANGLES {
                        let curr_ang = min_angle + (iang as Real * dtheta);

                        // Note that I am taking the absolute value here, as the intensity should always be positive.
                        // In poorly behaved interpolations this may not be the case, but I am trying to avoid negative probabilities being introduced to the PDF. 
                        let intens = prob_spline.eval(curr_ang).abs();
                        probs.push(intens * dtheta * curr_ang.sin());
                        angles.push(curr_ang);
                    }
                    // Now, we will instead normalise with our accumulator.
                    // The integrated intensity is not accurate enough, as it negates the sin theta term. 
                    let norm_factor: Real = probs.iter().sum();
                    probs = probs.into_iter()
                        .map(|val| val / norm_factor )
                        .collect();
                    
                } else {
                    // Apple the Sin Theta term to the angles / probabilities that are well enough sampled.
                    probs = angles.iter()
                        .zip(probs)
                        .map(|(ang, prob)| {
                            ang * prob.sin()
                        })
                        .collect();
                }
                
                // Now load the calculated properties into our CDF Plane.
                let mut curr_cdf_plane = SphericalCdfPlane::new();
                *curr_cdf_plane.azimuth_angle_mut() = plane.angle();
                *curr_cdf_plane.delta_aziumuth_mut() = photweb.delta_angle(iplane);

                // Construct the CDF for this plane using the probabilities and values that we have already extracted.
                *curr_cdf_plane.cdf_mut() = Probability::new_linear_spline(
                    &Array1::from(angles),
                    &Array1::from(probs),
                );

                // Now we just add on the upper edge of each of the planes. 
                azim_angles.push(*curr_cdf_plane.azimuth_angle());
                azim_probs.push(plane_intensity / total_intens);

                curr_cdf_plane
            })
            .collect::<Vec<SphericalCdfPlane>>();
        
        azim_angles.push(photweb.planes()[0].angle() + 2.0 * PI);
        azim_probs.push(photweb.planes()[0].integrate_intensity() / total_intens);
        
        // Load the finalised variables into the CDF.
        *cdf.planes_mut() = cdf_planes;
        *cdf.azimuth_cdf_mut() = Probability::new_linear_spline(&Array1::from(azim_angles), &Array1::from(azim_probs));
        cdf
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{
        math::Probability,
        data::Average,
    };
    use std::{
        f64::consts::PI,
        io::Write
    };
    use super::{ SphericalCdf, SphericalCdfPlane };
    use lidrs::photweb::{PhotometricWeb, Plane};
    use assert_approx_eq::assert_approx_eq;
    use ndarray::Array1;

    /// Tests that when we create an isotropic CDF we end up with a consistent outputs distribution
    /// from the sampling. 
    #[test]
    fn spherical_cdf_isotropic_test() {
        let planes = (0..360)
            .step_by(10)
            .enumerate()
            .map(|(_ipl, ang)| {
            let mut plane = Plane::new();
        
            // Iterate through the surfaces in the current plane to get the spline points for the CDF.
            let mut intens = vec![];
            let mut angles = vec![];

            for (_, ang) in (0..190).step_by(10).enumerate() {
                intens.push(1.0);
                angles.push(ang as f64);
            }
            
            plane.set_angles_degrees(&angles);
            plane.set_intensities(intens);
            plane.set_angle_degrees(ang as f64);
            plane.set_units(lidrs::photweb::IntensityUnits::Candela);
            plane.set_orientation(lidrs::photweb::PlaneOrientation::Vertical);
            plane
        })
        .collect();

        let mut photweb = PhotometricWeb::new();
        photweb.set_planes(planes);
        
        // Check that the web has been correctly assembled. 
        assert_eq!(photweb.n_planes(), 36);

        // Convert from photometric web to spherical CDF and check that the planes made it across. 
        let cdf: SphericalCdf = photweb.into();
        assert_eq!(cdf.planes().iter().count(), 36);

        // Output to file for analysis.
        cdf.azimuth_cdf().cdf_to_file("azim.cdf").unwrap();
        cdf.azimuth_cdf().pdf_to_file("azim.pdf").unwrap();
        for (ipl, pl) in cdf.planes().iter().enumerate() {
            let _ = pl.cdf().cdf_to_file(&format!("plane{}.cdf", ipl));
            let _ = pl.cdf().pdf_to_file(&format!("plane{}.pdf", ipl));
        }

        // Now sample the distribution. 
        let mut rng = rand::thread_rng();
        let mut az_ave = Average::new();
        let mut pol_ave = Average::new();
        
        //let mut samples_file = std::fs::File::create("samples.dat").unwrap();
        for _ in 0..10_000 {
            let (az, pol) = cdf.sample(&mut rng);
            az_ave += az;
            pol_ave += pol;
            //let _ = writeln!{samples_file, "{}\t{}", az, pol};
        }

        // Check that the average is correct given the input planes.  
        assert_approx_eq!(az_ave.ave() * (180. / PI), 180.0, 2.0);

        // Check that the average is correct given the input planes.  
        assert_approx_eq!(pol_ave.ave(), PI / 2.0, 0.1);
    }
    /*
    /// Tests that when we create a CDF with all probability concentrated in the lower hemisphere
    /// the output distribution of reflective of that. In this case, we want to check that all 
    /// photons are emitted from the lower half of the hemisphere (polar angle < PI / 2 radians).
    #[test]
    fn spherical_cdf_hemisphere_test() {

        let mut azim_probs = vec![];
        let mut azim_angs = vec![];
        let mut azim_accum = 0.0;

        let planes = (0..360)
            .step_by(90)
            .enumerate()
            .map(|(ipl, ang)| {
            let mut plane = SphericalCdfPlane::new();
        
            // Iterate through the surfaces in the current plane to get the spline points for the CDF.
            let mut plane_accum = 0.0;
            let mut probs = vec![];
            let mut angles = vec![];

            for (_, ang) in (0..190).step_by(10).enumerate() {
                probs.push(plane_accum);
                plane_accum += if ang <= 90 { 1.0 / 10.0 } else { 0.0 };
                angles.push(ang as f64 * (PI / 180.));
            }
            
            *plane.azimuth_angle_mut() = ang as f64 * (std::f64::consts::PI / 180.0);
            *plane.delta_aziumuth_mut() = std::f64::consts::PI / 2.0;

            // Construct this plane constibution to the azimuthal CDF here as we are iterating through. 
            if ipl == 0 {
                // The first plane will constribute two spline points - this is for the lower edge of the first plane.
                azim_angs.push(plane.azimuth_angle() - (plane.delta_aziumuth() / 2.0));
                azim_probs.push(azim_accum);
            }
            // Now we just add on the upper edge of each of the planes. 
            azim_accum += 1.0 / 4.0;
            azim_angs.push(plane.azimuth_angle() + (plane.delta_aziumuth() / 2.0));
            azim_probs.push(azim_accum);

            plane
        })
        .collect();
        
        let mut cdf = SphericalCdf::new();
        *cdf.planes_mut() = planes;
        *cdf.azimuth_cdf_mut() = Probability::new_linear_spline(&Array1::from(azim_probs), &Array1::from(azim_angs));

        // Now sample the distribution. 
        let mut rng = rand::thread_rng();
        let mut az_ave = Average::new();

        for _ in 0..10_000 {
            let (az, pol) = cdf.sample(&mut rng);
            // Check that all samples lie in the lower hemisphere.
            assert!(pol <= std::f64::consts::PI);
            az_ave += az;
        }

        // Check that the average is correct given the input planes.  
        assert_approx_eq!(az_ave.ave() * (180. / PI), (315. - 45.) / 2.0, 2.0);
    }

    /// This test checks that we can emit consistently from two connical sections. 
    /// In this case, the conical sections are emitting from the upper and lower
    /// 45 degrees of the distribution, hence there should be no photons outside of this.
    /// in addition, the polar average should remain consistent with the isotropic case. 
    #[test]
    fn spherical_cdf_connical_test() {

        let mut azim_probs = vec![];
        let mut azim_angs = vec![];
        let mut azim_accum = 0.0;

        let planes = (0..360)
            .step_by(90)
            .enumerate()
            .map(|(ipl, ang)| {
            let mut plane = SphericalCdfPlane::new();
        
            // Iterate through the surfaces in the current plane to get the spline points for the CDF.
            let mut plane_accum = 0.0;
            let mut probs = vec![];
            let mut angles = vec![];

            for (_, ang) in (0..185).step_by(5).enumerate() {
                plane_accum += if ang <= 45 || ang >= 135 { 1.0 / 19.0 } else { 0.0 };
                probs.push(plane_accum);
                angles.push(ang as f64 * (PI / 180.));
            }
            
            *plane.cdf_mut() = Probability::new_linear_spline(&Array1::from(probs), &Array1::from(angles));
            *plane.azimuth_angle_mut() = ang as f64 * (std::f64::consts::PI / 180.0);
            *plane.delta_aziumuth_mut() = std::f64::consts::PI / 2.0;

            // Construct this plane constibution to the azimuthal CDF here as we are iterating through. 
            if ipl == 0 {
                // The first plane will constribute two spline points - this is for the lower edge of the first plane.
                azim_angs.push(plane.azimuth_angle() - (plane.delta_aziumuth() / 2.0));
                azim_probs.push(azim_accum);
            }
            // Now we just add on the upper edge of each of the planes. 
            azim_accum += 1.0 / 4.0;
            azim_angs.push(plane.azimuth_angle() + (plane.delta_aziumuth() / 2.0));
            azim_probs.push(azim_accum);

            plane
        })
        .collect();
        
        let mut cdf = SphericalCdf::new();
        *cdf.planes_mut() = planes;
        *cdf.azimuth_cdf_mut() = Probability::new_linear_spline(&Array1::from(azim_probs), &Array1::from(azim_angs));

        // Now sample the distribution. 
        let mut rng = rand::thread_rng();
        let mut az_ave = Average::new();
        let mut pol_ave = Average::new();

        for _ in 0..10_000 {
            let (az, pol) = cdf.sample(&mut rng);
            az_ave += az;
            pol_ave += pol;
            

            // Check that the polar samples are within the conical sections. 
            assert!(pol <= PI / 4.0 || pol >= 3.0 * PI / 4.0)
        }

        // Check that the average is correct given the input planes.  
        assert_approx_eq!(az_ave.ave() * (180. / PI), (315. - 45.) / 2.0, 2.0);

        // Check that the average is correct given the input planes.  
        assert_approx_eq!(pol_ave.ave(), PI / 2.0, 0.2);
    }

    #[test]
    fn spherical_cdf_quadrant_test() {

        let mut azim_probs = vec![];
        let mut azim_angs = vec![];

        let planes = (0..360)
            .step_by(90)
            .enumerate()
            .map(|(ipl, ang)| {
            let mut plane = SphericalCdfPlane::new();
        
            // Iterate through the surfaces in the current plane to get the spline points for the CDF.
            let mut probs = vec![];
            let mut angles = vec![];

            for (_, ang) in (0..190).step_by(10).enumerate() {
                probs.push(1.0 / 18.0);
                angles.push(ang as f64 * (PI / 180.));
            }
            
            *plane.cdf_mut() = Probability::new_linear_spline(&Array1::from(angles), &Array1::from(probs));
            *plane.azimuth_angle_mut() = ang as f64 * (std::f64::consts::PI / 180.0);
            *plane.delta_aziumuth_mut() = std::f64::consts::PI / 2.0;

            // Construct this plane constibution to the azimuthal CDF here as we are iterating through. 
            if ipl == 0 {
                // The first plane will constribute two spline points - this is for the lower edge of the first plane.
                azim_angs.push(plane.azimuth_angle() - (plane.delta_aziumuth() / 2.0));
                azim_probs.push(0.0);
            }
            // Now we just add on the upper edge of each of the planes. 
            azim_angs.push(plane.azimuth_angle() + (plane.delta_aziumuth() / 2.0));
            azim_probs.push(if ang == 90 { 1.0 }  else { 0.0 });

            plane
        })
        .collect();
        
        let mut cdf = SphericalCdf::new();
        *cdf.planes_mut() = planes;
        *cdf.azimuth_cdf_mut() = Probability::new_linear_spline(&Array1::from(azim_angs), &Array1::from(azim_probs));

        // output to file for analysis.
        cdf.azimuth_cdf().cdf_to_file("azim.cdf").unwrap();
        cdf.azimuth_cdf().pdf_to_file("azim.pdf").unwrap();
        for (ipl, pl) in cdf.planes().iter().enumerate() {
            let _ = pl.cdf().cdf_to_file(&format!("plane{}.cdf", ipl));
            let _ = pl.cdf().pdf_to_file(&format!("plane{}.pdf", ipl));
        }

        // Now sample the distribution. 
        let mut rng = rand::thread_rng();
        let mut az_ave = Average::new();
        let mut pol_ave = Average::new();

        for _ in 0..10_000 {
            let (az, pol) = cdf.sample(&mut rng);
            // Check that all samples lie in the lower hemisphere.
            az_ave += az;
            pol_ave += pol;
        }

        // Check that the average is correct given the input planes.  
        assert_approx_eq!(az_ave.ave(), PI / 2.0, 0.2);

        // Check that the average is correct given the input planes.  
        assert_approx_eq!(pol_ave.ave(), PI / 2.0, 0.2);
    }
    */
}
