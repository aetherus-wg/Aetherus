//! Standard photon-lifetime engine function.

use crate::{
    io::output::{self, Output},
    phys::Photon,
    sim::{Attribute, Event, Input, scatter::scatter, surface::surface, travel::travel},
};
use rand::{Rng, RngExt};
use std::sync::{Arc, Mutex};

use aetherus_events::{EventType, SrcId, ledger::Ledger};

/// Simulate the life of a single photon.
#[allow(clippy::expect_used)]
pub fn standard<R: Rng>(
    input: &Input<(Attribute, SrcId)>,
    data: &mut Output,
    ledger: &Arc<Mutex<Ledger>>,
    mut rng: &mut R,
    mut phot: Photon,
) {
    // Add to the emission variables in which the photon is present.
    for vol in data.get_volumes_for_param_mut(output::OutputParameter::Emission) {
        if let Some(index) = vol.gen_index(phot.ray().pos()) {
            vol.data_mut()[index] += phot.power() * phot.weight();
        }
    }

    // Common constants.
    let bump_dist = input.sett.bump_dist();
    let loop_limit = input.sett.loop_limit();
    let min_weight = input.sett.min_weight();
    let roulette_barrels = input.sett.roulette_barrels() as f64;
    let roulette_survive_prob = 1.0 / roulette_barrels;

    // Initialisation.
    let mat = input.light.mat();
    let mut env = mat.sample_environment(phot.wavelength());
    // scat_dist persists across voxels propagation, in order to preserve scattering statistics
    let mut scat_dist = None;

    // Main event loop.
    let mut num_loops = 0;
    // It could be that this is preventing our photon packets from interacting with the boundary.
    //while let Some((index, voxel)) = input.grid.gen_index_voxel(phot.ray().pos()) {
    while input.bound.contains(phot.ray().pos()) {
        // Loop limit check.
        if num_loops >= loop_limit {
            println!("[WARN] : Terminating photon: loop limit reached.");
            break;
        }
        num_loops += 1;

        // Roulette.
        if phot.weight() < min_weight {
            let r = rng.random::<f64>();
            if r > roulette_survive_prob {
                break;
            }
            *phot.weight_mut() *= roulette_barrels;
        }

        // Scattering distance
        if scat_dist.is_none() {
            scat_dist = Some(-(rng.random::<f64>()).ln() / env.scat_coeff());
        }
        // NOTE: Does aggregated absorption and scattering reduce the scanning depth?
        // Perhaps the distance considerations significantly reduces the time to complete the Tree search
        let surf_hit = input.tree.scan(
            phot.ray().clone(),
            bump_dist,
            scat_dist.unwrap()
        );
        let boundary_hit = input.bound
            .dist_boundary(phot.ray())
            .expect("Photon not contained in boundary.");

        let interaction_event = Event::new(scat_dist.unwrap(), surf_hit, boundary_hit, bump_dist);

        data.volume_estimate(&env, &phot, interaction_event.dist(), bump_dist);

        // Event handling.
        match interaction_event {
            Event::Scattering(dist) => {
                travel(&mut phot, &env, dist);
                let event_id = scatter(&mut rng, &mut phot, &env);
                // WARN: Accessing Ledger from many threads will slow down the simulation
                // considerably. Consider using an async design, encapsulating `uid` in work token
                // and computed value, transforming insert into a send on an mpsc channel
                if input.sett.uid_tracked() == Some(true) {
                    *phot.uid_mut() = ledger.lock()
                                            .expect("Can't lock Ledger")
                                            .insert(phot.uid(), event_id);

                }
                // Reset scat_dist for the new scattering event
                scat_dist = None;
            }
            Event::Surface(hit) => {
                travel(&mut phot, &env, hit.dist());
                // FIXME: next_seq_id needed here only for PhotonCollector, which needs the updated
                // Uid before it can store the photon data. How to solve this RAW hazard?
                let next_seq_id = ledger.lock().expect("Can't lock Ledger").get_next_seq_id(&phot.uid());
                let prev_env = env.clone();
                let event_id = surface(&mut rng, &hit, &mut phot, &mut env, data, next_seq_id);
                phot.ray_mut().travel(bump_dist);
                if input.sett.uid_tracked() == Some(true) && event_id.event_type != EventType::None {
                    *phot.uid_mut() = ledger.lock()
                                            .expect("Can't lock Ledger")
                                            .insert(phot.uid(), event_id);
                }

                // FIXME: Is the surface interaction also affecting the scattering statistics like
                // voxels did? => Based on "MONTE CARLO MODELLING OF PHOTON TRANSPORT IN BIOLOGICAL TISSUE - p40" yes
                // Recompute the scattering distance for the new env
                scat_dist = Some(scat_dist.unwrap() - hit.dist());
                assert!(scat_dist.unwrap() >= 0.0);
            },
            Event::Boundary(boundary_hit) => {
                travel(&mut phot, &env, boundary_hit.dist());
                input.bound.apply(rng, &boundary_hit, &mut phot);
                // Allow for the possibility that the photon got killed at the boundary - hence don't evolve.
                if phot.weight() > 0.0 {
                    travel(&mut phot, &env, bump_dist);
                    scat_dist = Some(scat_dist.unwrap() - boundary_hit.dist()- bump_dist);
                }
            }
        }

        if phot.weight() <= 0.0 {
            break;
        }
    }
}
