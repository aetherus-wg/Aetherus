//! Photon movement function.

use crate::{
    phys::{Local, Photon},
    io::output::{Output, OutputParameter},
};
use physical_constants::SPEED_OF_LIGHT_IN_VACUUM;

/// Move the photon forward and record the flight.
#[inline]
pub fn travel(data: &mut Output, phot: &mut Photon, env: &Local, dist: f64) {
    debug_assert!(dist > 0.0);

    // Energy Density.
    let weight_power_dist = phot.weight() * phot.power() * dist;
    for vol in data.get_volumes_for_param_mut(OutputParameter::Energy) {
        if let Some(index) = vol.gen_index(phot.ray().pos()) {
            vol.data_mut()[index] += weight_power_dist * env.ref_index() / SPEED_OF_LIGHT_IN_VACUUM;
        }
    }

    // Absorption.
    for vol in data.get_volumes_for_param_mut(OutputParameter::Absorption) {
        if let Some(index) = vol.gen_index(phot.ray().pos()) {
            vol.data_mut()[index] += weight_power_dist * env.abs_coeff();
        }
    }

    // Shifts.
    for vol in data.get_volumes_for_param_mut(OutputParameter::Shift) {
        if let Some(index) = vol.gen_index(phot.ray().pos()) {
            vol.data_mut()[index] += weight_power_dist * env.shift_coeff();
        }
    }

    phot.ray_mut().travel(dist);

    // Update time of flight.
    match phot.tof() {
        // TODO: Precompute (1 / SPEED_OF_LIGHT_IN_VACUUM) and use MUL
        // instead of DIV operation, for better performance. Benchmark first!
        Some(tof) => {
            *phot.tof_mut() = Some(tof + dist * env.ref_index() / SPEED_OF_LIGHT_IN_VACUUM);
        },
        None => (),
    };
}
