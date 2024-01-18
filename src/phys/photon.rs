//! Photon particle.

use crate::{access, clone, geom::Ray, math::{Dir3, Point3}};

/// Photon.
#[derive(Clone)]
pub struct Photon {
    /// Ray of travel.
    ray: Ray,
    /// Statistical weight.
    weight: f64,
    /// Wavelength (m).
    wavelength: f64,
    /// Power (J/s).
    power: f64,
}

impl Photon {
    access!(ray, ray_mut: Ray);
    clone!(weight, weight_mut: f64);
    clone!(wavelength, wavelength_mut: f64);
    clone!(power: f64);

    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(ray: Ray, wavelength: f64, power: f64) -> Self {
        debug_assert!(wavelength > 0.0);
        debug_assert!(power > 0.0);

        Self {
            ray,
            weight: 1.0,
            wavelength,
            power,
        }
    }

    /// Set the weight to zero.
    #[inline]
    pub fn kill(&mut self) {
        self.weight = 0.0;
    }
}


/// Photon reconstructed into raw data for MPI buffer.
#[derive(Clone)]
pub struct PhotonBuf {
    /// Ray of travel broken down to component arrays
    pub ray_pos: [f64; 3],
    pub ray_dir: [f64; 3],
    /// Statistical weight.
    pub weight: f64,
    /// Wavelength (m).
    pub wavelength: f64,
    /// Power (J/s).
    pub power: f64,
}

impl PhotonBuf {

    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(photon: Photon) -> Self {
        Self {
            ray_pos: [photon.ray().pos().x(), photon.ray().pos().y(), photon.ray().pos().z()],
            ray_dir: [photon.ray().dir().x(), photon.ray().dir().y(), photon.ray().dir().z()],
            weight: photon.weight(),
            wavelength: photon.wavelength(),
            power: photon.power(),
        }
    }

    /// Convert photon buffer back to Photon struct
    #[inline]
    pub fn as_photon(self) -> Photon {
        let ray = Ray::new(
            Point3::new(self.ray_pos[0], self.ray_pos[1], self.ray_pos[2]),
            Dir3::new(self.ray_dir[0], self.ray_dir[1], self.ray_dir[2]));
        return Photon::new(ray, self.wavelength, self.power);
    }

    

}
