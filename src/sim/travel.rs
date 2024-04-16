//! Photon movement function.

use crate::{
    phys::{Local, Photon},
    sim::Output,
    geom::Cube,
};
use physical_constants::SPEED_OF_LIGHT_IN_VACUUM;

/// Move the photon forward and record the flight.
#[inline]
pub fn travel(data: &mut Output, phot: &mut Photon, env: &Local, index: &Option<([usize; 3], Cube)>, dist: f64) {
    debug_assert!(dist > 0.0);

    match *index {
        Some((idx, _)) => {
            let weight_power_dist = phot.weight() * phot.power() * dist;
            data.energy[idx] += weight_power_dist * env.ref_index() / SPEED_OF_LIGHT_IN_VACUUM;
            data.absorptions[idx] += weight_power_dist * env.abs_coeff();
            data.shifts[idx] += weight_power_dist * env.shift_coeff();
        },
        None => {},
    }

    phot.ray_mut().travel(dist);
}
