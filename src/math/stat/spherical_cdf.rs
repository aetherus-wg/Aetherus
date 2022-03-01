use crate::{
    access, clone, core::Real, math::stat::CumulativeDistributionFunction, ord::Spherical,
};
use dimensioned::{si::L, typenum::False};
use lidrs::photweb::{PhotometricWeb, Plane};
use rand::Rng;
use serde::{Serialize, Deserialize};
use std::default::Default;
use std::f64::consts::PI;

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

                for (isurf, intens) in plane.intensities().iter().enumerate() {
                    plane_accum += (intens * plane.delta_angle(isurf) * plane.width() * plane.angles()[isurf].sin()) / plane_intensity;
                    probs.push(plane_accum);
                    angles.push(plane.angles()[isurf]);
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
pub mod tests {}
