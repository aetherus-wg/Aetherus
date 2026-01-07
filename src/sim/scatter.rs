//! Photon scattering function.

use crate::{
    math::sample_henyey_greenstein,
    phys::{Local, Photon},
};
use aetherus_events::{EventId, EventType, mcrt_event};
use rand::{Rng, RngExt};
use std::f64::consts::PI;

/// Perform a photon scattering event.
pub fn scatter<R: Rng>(rng: &mut R, phot: &mut Photon, env: &Local) -> EventId {
    let phi = sample_henyey_greenstein(rng, env.asym());
    let theta = rng.random_range(0.0..(PI * 2.0));
    phot.ray_mut().rotate(phi, theta);
    EventId { event_type: EventType::MCRT(mcrt_event!(Material, Elastic, HenyeyGreenstein, Any)), src_id: *env.mat_id() }
}

/// Perform a photon scattering event with a probability of shifting wavelength.
pub fn shift_scatter<R: Rng>(rng: &mut R, phot: &mut Photon, env: &Local) -> EventId {
    // The remaining weight may be shifted in a Raman/fluorescence event.
    let r = rng.random::<f64>();
    if r <= env.shift_prob() {
        // Shift occurs.
        // Fluorescence event removes photons from optical range of interest.
        *phot.weight_mut() = 0.0;
        return EventId { event_type: EventType::MCRT(mcrt_event!(Material, Inelastic, Raman, Any)), src_id: *env.mat_id() }
    }

    // The remaining weight is scattered.
    let phi = sample_henyey_greenstein(rng, env.asym());
    let theta = rng.random_range(0.0..(PI * 2.0));
    phot.ray_mut().rotate(phi, theta);

    EventId { event_type: EventType::MCRT(mcrt_event!(Material, Inelastic, Raman, Any)), src_id: *env.mat_id() }
}
