//! Photon movement function.

use crate::{
    phys::{Local, Photon},
};

/// Move the photon forward and record the flight.
pub fn travel(phot: &mut Photon, env: &Local, dist: f64) {
    debug_assert!(dist > 0.0);

    phot.ray_mut().travel(dist);

    // Update weight based on Beer's Law
    *phot.weight_mut() *= (-env.abs_coeff() * dist).exp();
}
