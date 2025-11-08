//! Photon particle.
use crate::{access, clone, geom::Ray};

#[cfg(feature = "mpi")]
use crate::math::{Dir3, Point3};
#[cfg(feature = "mpi")]
use mpi::{
    datatype::{UncommittedUserDatatype, UserDatatype},
    traits::*,
    Address,
};
#[cfg(feature = "mpi")]
use memoffset::offset_of;

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
#[cfg(feature = "mpi")]
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

#[cfg(feature = "mpi")]
impl PhotonBuf {

    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(photon: &Photon) -> Self {
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

#[cfg(feature = "mpi")]
unsafe impl Equivalence for PhotonBuf {
    type Out = UserDatatype;
    fn equivalent_datatype() -> Self::Out {
        UserDatatype::structured(
            &[1, 1, 1, 1, 1],
            &[
                offset_of!(PhotonBuf, ray_pos) as Address,
                offset_of!(PhotonBuf, ray_dir) as Address,
                offset_of!(PhotonBuf, weight) as Address,
                offset_of!(PhotonBuf, wavelength) as Address,
                offset_of!(PhotonBuf, power) as Address,
            ],
            &[
                UncommittedUserDatatype::contiguous(3, &f64::equivalent_datatype()).as_ref(),
                UncommittedUserDatatype::contiguous(3, &f64::equivalent_datatype()).as_ref(),
                f64::equivalent_datatype().into(),
                f64::equivalent_datatype().into(),
                f64::equivalent_datatype().into(),
            ],
        )
    }
}


#[cfg(test)]
mod tests {
    #[cfg(feature = "mpi")]
    use super::Photon;
    #[cfg(feature = "mpi")]
    use crate::geom::Ray;
    #[cfg(feature = "mpi")]
    use crate::math::{Dir3, Point3};
    #[cfg(feature = "mpi")]
    use assert_approx_eq::assert_approx_eq;
    #[cfg(feature = "mpi")]
    use std::f64;

    #[cfg(feature = "mpi")]
    use super::PhotonBuf;

    /// Check that the creation and accessing code is working correctly.
    #[test]
    #[cfg(feature = "mpi")]
    fn buf_init_test() {
        let ray = Ray::new(Point3::new(0.0, 0.0, 0.0), Dir3::new(1.0, 1.0, 1.0));
        let phot = Photon::new(ray, 500.0, 10.0);

        let phot_buf  = PhotonBuf::new(&phot);
        // Check that we get the correct
        assert_eq!(phot_buf.ray_pos, [0.0, 0.0, 0.0]);
        //assert_eq!(phot_buf.ray_dir, [1.0, 1.0, 1.0]);
        assert_eq!(phot_buf.weight, 1.0);
        assert_eq!(phot_buf.wavelength, 500.0);
        assert_eq!(phot_buf.power, 10.0);
    }

    /// Check that arrays destruct correctly
    #[test]
    #[cfg(feature = "mpi")]
    fn buf_as_photon_test() {
        let ray = Ray::new(Point3::new(0.0, 0.0, 0.0), Dir3::new(1.0, 1.0, 1.0));
        let phot = Photon::new(ray, 500.0, 10.0);
        let phot_buf = PhotonBuf::new(&phot);

        let phot_return = phot_buf.as_photon();

        assert_eq!(phot.ray.pos(), phot_return.ray.pos());
        assert_eq!(phot.ray.dir(), phot_return.ray.dir());
        assert_eq!(phot.weight(), phot_return.weight());
        assert_eq!(phot.wavelength(), phot_return.wavelength());
        assert_eq!(phot.power(), phot_return.power());
    }

}