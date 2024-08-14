//! Standard photon-lifetime engine function.

use crate::{
    geom::Trace, io::output::{self, Output}, phys::Photon, sim::{scatter::scatter, surface::surface, travel::travel, Event, Input}
};
use rand::{rngs::ThreadRng, Rng};

/// Simulate the life of a single photon.
#[allow(clippy::expect_used)]
#[inline]
pub fn standard(input: &Input, mut data: &mut Output, mut rng: &mut ThreadRng, mut phot: Photon) {
    // Add the emission to output volumes. 
    // if let Some(index) = input.grid.gen_index(phot.ray().pos()) {
    //     data.emission[index] += phot.power() * phot.weight();
    // } else {
    //     panic!("Photon was not emitted within the grid.");
    // }

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
            let r = rng.gen::<f64>();
            if r > roulette_survive_prob {
                break;
            }
            *phot.weight_mut() *= roulette_barrels;
        }

        // Interaction distances.
        let index = input.grid.gen_index_voxel(phot.ray().pos());
        let voxel_dist = match &index {
            Some((_index, voxel)) => {
                voxel.dist(phot.ray()).expect("Could not determine voxel distance.")
            },
            None => f64::INFINITY,
        };
        let scat_dist = -(rng.gen::<f64>()).ln() / env.inter_coeff();
        let surf_hit = input
            .tree
            .scan(phot.ray().clone(), bump_dist, voxel_dist.min(scat_dist));
        let boundary_hit = input.bound.dist_boundary(phot.ray()).expect("Photon not contained in boundary. ");

        // Event handling.
        match Event::new(voxel_dist, scat_dist, surf_hit, boundary_hit, bump_dist) {
            Event::Voxel(dist) => travel(&mut data, &mut phot, &env, dist + bump_dist),
            Event::Scattering(dist) => {
                travel(&mut data, &mut phot, &env,dist);
                scatter(&mut rng, &mut phot, &env);
            }
            Event::Surface(hit) => {
                travel(&mut data, &mut phot, &env,hit.dist());
                surface(&mut rng, &hit, &mut phot, &mut env, &mut data);
                travel(&mut data, &mut phot, &env,bump_dist);
            },
            Event::Boundary(boundary_hit) => {
                travel(&mut data, &mut phot, &env, boundary_hit.dist());
                input.bound.apply(rng, &boundary_hit, &mut phot);
                travel(&mut data, &mut phot, &env, 100.0 * bump_dist);
            }
        }

        if phot.weight() <= 0.0 {
            break;
        }
    }
}
