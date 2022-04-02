use crate::{
    access, clone, core::Real, math::stat::CumulativeDistributionFunction, ord::Spherical,
};
use dimensioned::{si::L, typenum::False};
use lidrs::photweb::{PhotometricWeb, Plane};
use rand::Rng;
use serde::{Serialize, Deserialize};
use statrs::statistics::Statistics;
use std::default::Default;
use std::f64::consts::PI;

/// This is the target number of polar angles that we are aiming for for the spherical CDF.
/// If more than this, we will not interpolate, however, if less we will interpolate data points
/// to ensure that we can sample the sin(theta) area term well enough. 
const TARGET_NANGLES: usize = 360;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct SphericalCdfPlane {
    /// The central azimurhal angle of the plane.
    azimuth_angle: Real,
    /// The angular diameter of the plane in the azimuthal axis.
    delta_aziumuth: Real,
    cdf: CumulativeDistributionFunction,
}

impl SphericalCdfPlane {
    access!(azimuth_angle, azimuth_angle_mut: Real);
    access!(delta_aziumuth, delta_aziumuth_mut: Real);
    access!(cdf, cdf_mut: CumulativeDistributionFunction);

    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Checks to see if the azimuthal angle is contained within the curren plane.
    /// This will return true if the angle is in the plane, else it will return false.
    pub fn azimuthal_angle_in_plane(&self, azimuthal_angle: Real) -> bool {
        let half_dazimuth = self.delta_aziumuth / 2.0;

        let mut offset_angle = 0.0;
        // First do check to see if there are going to be any problems with being around the zero / 2PI point,
        // If so, then apply an offset so that this should no longer be an issue.
        if self.azimuth_angle - half_dazimuth < 0.0 {
            offset_angle = -(self.azimuth_angle - half_dazimuth);
        }
        if self.azimuth_angle + half_dazimuth > 2.0 * PI {
            offset_angle = -(self.azimuth_angle + half_dazimuth - 2.0 * PI);
        }

        if azimuthal_angle + offset_angle >= self.azimuth_angle + offset_angle - half_dazimuth
            && azimuthal_angle + offset_angle < self.azimuth_angle + offset_angle + half_dazimuth
        {
            true
        } else {
            false
        }
    }

    /// Sample the CDF of this plane to return an angle consistent with this CDF.
    pub fn sample<R: Rng>(&self, rng: &mut R) -> Real {
        self.cdf.sample(rng)
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
/// The spherical CDF object.
pub struct SphericalCdf {
    planes: Vec<SphericalCdfPlane>,
    azimuth_cdf: CumulativeDistributionFunction,
}

impl SphericalCdf {
    access!(planes, planes_mut: Vec<SphericalCdfPlane>);
    access!(azimuth_cdf, azimuth_cdf_mut: CumulativeDistributionFunction);

    /// Returns a new default spherical cumulative distribution function.
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Returns true if the distribution is spherically symmetric - in this case there will only be one plane.
    pub fn is_spherically_symmetric(&self) -> bool {
        self.planes.iter().count() == 1
    }

    /// Samples the CDF and returns a tuple containing the azimuthal and polar angles in radians.
    /// These angles are randomly chosen based on the underlying CDF.
    pub fn sample<R: Rng>(&self, rng: &mut R) -> (Real, Real) {
        // First, draw the azimuthal angle from the azimuthal CDF.
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
    /// The main functiont the does the conversion from `lidrs::photweb::PhotometricWeb` to a spherical
    /// CDF that Aetherus can understand.
    fn from(photweb: PhotometricWeb) -> SphericalCdf {
        let mut cdf = SphericalCdf::new();

        let mut azim_angles: Vec<Real> = vec![];
        let mut azim_probs: Vec<Real> = vec![];
        let mut azim_accum = 0.0;

        // Calculate the azimuth angles.
        let total_intens = photweb.total_intensity();

        // Iterate through the planes in the photometric web and convert them to CDFs for each plane.
        let cdf_planes = photweb
            .planes()
            .iter()
            .enumerate()
            .map(|(iplane, plane)| {
                // First calculate the normalised probabilities for reach of the angles - this is our PDF.
                let plane_intensity = plane.integrate_intensity();
                
                // Iterate through the surfaces in the current plane to get the spline points for the CDF.
                let mut plane_accum = 0.0;
                let mut probs = vec![];
                let mut angles = vec![];
                let mut unnorm_probs = vec![];

                if true {
                    for (isurf, intens) in plane.intensities().iter().enumerate() {
                        plane_accum += (intens * plane.delta_angle(isurf) * plane.width() * plane.angles()[isurf].sin()) / plane_intensity;
                        probs.push(plane_accum);
                        angles.push(plane.angles()[isurf]);
                    }
                } else {
                    // Interpolate this term.
                    let min_angle = plane.angles().min();
                    let max_angle = plane.angles().max();
                    let dtheta = (max_angle - min_angle) / (TARGET_NANGLES as Real);
                    for iang in 0..TARGET_NANGLES {
                        let curr_ang = min_angle + (iang as Real * dtheta);

                        // Find the surface we are interested in. 
                        let isurf = plane.angles().iter()
                            .position(|theta| {
                                curr_ang <= *theta
                            }).unwrap();

                        let intens = plane.intensities()[isurf];
                        plane_accum += (intens * dtheta * curr_ang.sin());
                        unnorm_probs.push(plane_accum);
                        angles.push(curr_ang);
                    }
                    // Now, we will instead normalise with our accumulator.
                    // The integrated intensity is not accurate enough, as it negates the sin theta term. 
                    // TODO: If you think you can do a better job than this in the future, please do. 
                    probs = unnorm_probs.iter()
                        .map(|val| *val / plane_accum)
                        .collect();
                }
                
                // Now load the calculated properties into our CDF Plane.
                let mut curr_cdf_plane = SphericalCdfPlane::new();
                *curr_cdf_plane.azimuth_angle_mut() = plane.angle();
                *curr_cdf_plane.delta_aziumuth_mut() = photweb.delta_angle(iplane);

                // Construct the CDF for this plane using the probabilities and values that we have already extracted.
                *curr_cdf_plane.cdf_mut() = CumulativeDistributionFunction::from_spline_points(
                    probs,
                    angles,
                );

                // Construct this plane constibution to the azimuthal CDF here as we are iterating through. 
                if iplane == 0 {
                    // The first plane will constribute two spline points - this is for the lower edge of the first plane.
                    azim_angles.push(curr_cdf_plane.azimuth_angle() - (curr_cdf_plane.delta_aziumuth() / 2.0));
                    azim_probs.push(azim_accum);
                }
                // Now we just add on the upper edge of each of the planes. 
                azim_accum += plane_intensity / total_intens;
                azim_angles.push(curr_cdf_plane.azimuth_angle() + (curr_cdf_plane.delta_aziumuth() / 2.0));
                azim_probs.push(azim_accum);

                curr_cdf_plane
            })
            .collect::<Vec<SphericalCdfPlane>>();

        // Load the finalised variables into the CDF.
        *cdf.planes_mut() = cdf_planes;
        *cdf.azimuth_cdf_mut() = CumulativeDistributionFunction::from_spline_points(azim_probs, azim_angles);

        cdf
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{
        math::CumulativeDistributionFunction,
        data::Average,
    };
    use std::{
        fs::File,
        io::Write,
        f64::consts::PI
    };
    use serde_json::to_string_pretty;
    use super::{ SphericalCdf, SphericalCdfPlane };
    use assert_approx_eq::assert_approx_eq;

    /// Tests that when we create an isotropic CDF we end up with a consistent outputs distribution
    /// from the sampling. 
    #[test]
    fn spherical_cdf_isotropic_test() {

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
                plane_accum += 1.0 / 18.0;
                probs.push(plane_accum);
                angles.push(ang as f64 * (PI / 180.));
            }
            
            *plane.cdf_mut() = CumulativeDistributionFunction::from_spline_points(probs, angles);
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
        *cdf.azimuth_cdf_mut() = CumulativeDistributionFunction::from_spline_points(azim_probs, azim_angs);

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
        assert_approx_eq!(az_ave.ave() * (180. / PI), (315. - 45.) / 2.0, 2.0);

        // Check that the average is correct given the input planes.  
        assert_approx_eq!(pol_ave.ave(), PI / 2.0, 0.1);
    }

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
            
            *plane.cdf_mut() = CumulativeDistributionFunction::from_spline_points(probs, angles);
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
        *cdf.azimuth_cdf_mut() = CumulativeDistributionFunction::from_spline_points(azim_probs, azim_angs);

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
            
            *plane.cdf_mut() = CumulativeDistributionFunction::from_spline_points(probs, angles);
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
        *cdf.azimuth_cdf_mut() = CumulativeDistributionFunction::from_spline_points(azim_probs, azim_angs);

        let json_str = to_string_pretty(&cdf).unwrap();
        let mut file = std::fs::File::create("test.json").unwrap();
        let _ = write!(file, "{}", json_str);

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
                plane_accum += 1.0 / 18.0;
                angles.push(ang as f64 * (PI / 180.));
            }
            
            *plane.cdf_mut() = CumulativeDistributionFunction::from_spline_points(probs, angles);
            *plane.azimuth_angle_mut() = ang as f64 * (std::f64::consts::PI / 180.0);
            *plane.delta_aziumuth_mut() = std::f64::consts::PI / 2.0;

            // Construct this plane constibution to the azimuthal CDF here as we are iterating through. 
            if ipl == 0 {
                // The first plane will constribute two spline points - this is for the lower edge of the first plane.
                azim_angs.push(plane.azimuth_angle() - (plane.delta_aziumuth() / 2.0));
                azim_probs.push(azim_accum);
            }
            // Now we just add on the upper edge of each of the planes. 
            azim_accum += if ang == 90 { 1.0 }  else { 0.0 };
            azim_angs.push(plane.azimuth_angle() + (plane.delta_aziumuth() / 2.0));
            azim_probs.push(azim_accum);

            plane
        })
        .collect();
        
        let mut cdf = SphericalCdf::new();
        *cdf.planes_mut() = planes;
        *cdf.azimuth_cdf_mut() = CumulativeDistributionFunction::from_spline_points(azim_probs, azim_angs);

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
}
