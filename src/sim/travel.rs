//! Photon movement function.

use crate::{
    phys::{Local, Photon},
};
use physical_constants::SPEED_OF_LIGHT_IN_VACUUM;

/// Move the photon forward and record the flight.
pub fn travel(phot: &mut Photon, env: &Local, dist: f64) {
    debug_assert!(dist > 0.0);

    phot.ray_mut().travel(dist);

    // Update weight based on Beer's Law
    if env.abs_coeff() >= f64::MIN_POSITIVE {
        *phot.weight_mut() *= (-env.abs_coeff() * dist).exp();
    }

    // Update time of flight.
    if let Some(tof) = phot.tof_mut().as_mut() {
        // TODO: Precompute (1 / SPEED_OF_LIGHT_IN_VACUUM) and use MUL
        // instead of DIV operation, for better performance. Benchmark first!
        *tof += dist * env.ref_index() / SPEED_OF_LIGHT_IN_VACUUM;
    };
}
