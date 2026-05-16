//! Photon scattering function.

use crate::{
    geom::Hit,
    io::output::Output,
    phys::{Crossing, Local, Photon},
    sim::Attribute,
};
use aetherus_events::prelude::*;
use rand::{Rng, RngExt};

/// Handle a surface collision.
#[allow(clippy::expect_used)]
pub fn surface<R: Rng>(
    rng: &mut R,
    hit: &Hit<(Attribute, SrcId)>,
    phot: &mut Photon,
    env: &mut Local,
    data: &mut Output,
    seq_id: Option<u32>,
) -> EventId {
    match hit.tag().0 {
        Attribute::Interface(ref inside, ref outside) => {
            // Reference materials.
            let (curr_mat, next_mat) = if hit.side().is_inside() {
                (inside, outside)
            } else {
                (outside, inside)
            };

            // Find local optical environments.
            let curr_env = curr_mat.sample_environment(phot.wavelength());
            let next_env = next_mat.sample_environment(phot.wavelength());

            debug_assert!(
                curr_env.scat_coeff() == env.scat_coeff() && curr_env.ref_index() == env.ref_index() && curr_env.abs_coeff() == env.abs_coeff(),
                "Current env cached in the simulation doesn't match env detected from surface interaction: identified={:?} vs cached={:?}",
                curr_env, env
            );

            // Get the near, and far side refractive indices.
            let curr_ref_index = curr_env.ref_index();
            let next_ref_index = next_env.ref_index();

            // Calculate the crossing normal vectors.
            let crossing = Crossing::new(
                phot.ray().dir(),
                hit.side().norm(),
                curr_ref_index,
                next_ref_index,
            );

            // Determine if a reflection or transmission occurs.
            let r = rng.random::<f64>();
            if r <= crossing.ref_prob() {
                // Reflect.
                let new_dir = *crossing.ref_dir();
                let norm = hit.side().norm();
                debug_assert!(phot.ray().dir().dot(norm) * new_dir.dot(norm)>= 0.0, "Reflection direction is not on the correct side of the surface");
                phot.ray_mut().update_dir(new_dir);
                EventId { event_type: EventType::MCRT(mcrt_event!(Interface, Reflection)), src_id: hit.tag().1}
            } else {
                // Refract.
                let new_dir = crossing.trans_dir().expect("Invalid refraction.");
                debug_assert!(phot.ray().dir().dot(&new_dir) >= 0.0, "Refraction direction is not on the correct side of the surface");
                phot.ray_mut().update_dir(new_dir);
                *env = next_env;
                EventId { event_type: EventType::MCRT(mcrt_event!(Interface, Refraction)), src_id: env.mat_id() }
            }
        }
        Attribute::Reflector(ref reflectance) => {
            // NOTE: Instead of killing the photon based on reflection probability, reduce its weight
            match reflectance.reflect(rng, phot, hit) {
                Some((ray, ref_prob)) => {
                    *phot.ray_mut() = ray;
                    *phot.weight_mut() *= ref_prob;
                },
                None => phot.kill(),
            }
            // FIXME: Get reflector type from the reflectance.reflect() fn instead
            EventId { event_type: EventType::MCRT(mcrt_event!(Reflector, Specular)), src_id: hit.tag().1 }
        }
        Attribute::Mirror(abs) => {
            *phot.weight_mut() *= abs;
            let new_dir = Crossing::calc_ref_dir(phot.ray().dir(), hit.side().norm());
            phot.ray_mut().update_dir(new_dir);
            EventId { event_type: EventType::MCRT(mcrt_event!(Reflector, Specular)), src_id: hit.tag().1 }
        }
        Attribute::Detector(id) => {
            if !hit.side().is_inside() {
                // FIXME: The photon collection happens before the new Uid is updated in the
                // photon, hence this is a very ugly walk-around to fix this issues which needs to be
                // sorted out properly.
                let mut future_phot = phot.clone();
                let event_id = EventId { event_type: EventType::Detection, src_id: hit.tag().1 };
                if let Some(seq_id) = seq_id {
                    *future_phot.uid_mut() = Uid::from_event(seq_id, &event_id);
                }
                data.phot_cols[id].collect_photon(&mut future_phot);
                if future_phot.weight() <= 0.0 {
                    phot.kill();
                }
                event_id
            } else {
                EventId { event_type: EventType::None, src_id: hit.tag().1 }
            }
        }
        Attribute::AttributeChain(ref _attrs) => {
            // FIXME: For some reason this is not working
            //for attr in attrs {
            //    let hit_proxy = Hit::new(attr, hit.dist(), hit.side().clone());
            //    surface(rng, &hit_proxy, phot, env, data);
            //}
            EventId { event_type: EventType::None, src_id: hit.tag().1 }
        }
    }
}

