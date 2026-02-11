//! Raman photon-lifetime engine function.

use crate::{
    math::Point3,
    phys::Photon,
    sim::{scatter::shift_scatter, surface::surface, travel::travel, Event, Input},
    io::output::{Output, OutputParameter},
};
use rand::{rngs::ThreadRng, Rng};

/// Simulate the life of a single photon which has the potential to generate a Raman photon.
#[allow(clippy::expect_used)]
pub fn raman(
    _detector_pos: &Point3,
    input: &Input,
    mut data: &mut Output,
    mut rng: &mut ThreadRng,
    mut phot: Photon,
) {
    // Add to the emission variables in which the photon is present. 
    for vol in data.get_volumes_for_param_mut(OutputParameter::Emission) {
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
    // let mut detected_weight = 0.0;

    // Main event loop.
    let mut num_loops = 0;
    while input.bound.contains(phot.ray().pos()) {
        // Loop limit check.
        if num_loops >= loop_limit {
            println!("[WARN] : Terminating photon: loop limit reached.");
            break;
        }
        num_loops += 1;

        // Roulette.
        if phot.weight() < min_weight {
            let r = rng.gen::<f64>();
            if r > roulette_survive_prob {
                break;
            }
            *phot.weight_mut() *= roulette_barrels;
        }

        // Interaction distances.
        let voxel_dist = data.voxel_dist(&phot);
        let scat_dist = -(rng.gen::<f64>()).ln() / env.inter_coeff();
        let surf_hit = input
            .tree
            .scan(phot.ray().clone(), bump_dist, voxel_dist.min(scat_dist));
        let boundary_hit = input.bound.dist_boundary(phot.ray()).expect("Photon not contained in boundary. ");

        // Event handling.
        match Event::new(voxel_dist, scat_dist, surf_hit, boundary_hit, bump_dist) {
            Event::Voxel(dist) => travel(&mut data, &mut phot, &env, dist + bump_dist),
            Event::Scattering(dist) => {
                travel(&mut data, &mut phot, &env, dist);

                // // Capture.
                // if let Some(weight) =
                //     crate::sim::mcrt::peel_off::peel_off(input, phot.clone(), &env, *detector_pos)
                // {
                //     detected_weight += weight;
                // }

                shift_scatter(&mut rng, &mut phot, &env);
            }
            Event::Surface(hit) => {
                travel(&mut data, &mut phot, &env, hit.dist());
                surface(&mut rng, &hit, &mut phot, &mut env, &mut data);
                travel(&mut data, &mut phot, &env, bump_dist);
            },
            Event::Boundary(boundary_hit) => {
                travel(&mut data, &mut phot, &env, boundary_hit.dist());
                input.bound.apply(rng, &boundary_hit, &mut phot);
                // Allow for the possibility that the photon got killed at the boundary - hence don't evolve. 
                if phot.weight() > 0.0 {
                    travel(&mut data, &mut phot, &env, bump_dist);
                }
            }
        }

        if phot.weight() <= 0.0 {
            break;
        }
    }
}
