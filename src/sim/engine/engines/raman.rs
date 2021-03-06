//! Raman photon-lifetime engine function.

use crate::{
    geom::Trace,
    math::Point3,
    phys::Photon,
    sim::{scatter::shift_scatter, surface::surface, travel::travel, Event, Input, Output},
};
use rand::{rngs::ThreadRng, Rng};

/// Simulate the life of a single photon which has the potential to generate a Raman photon.
#[allow(clippy::expect_used)]
#[inline]
pub fn raman(
    _detector_pos: &Point3,
    input: &Input,
    mut data: &mut Output,
    mut rng: &mut ThreadRng,
    mut phot: Photon,
) {
    // Check photon is within the grid.
    if let Some(index) = input.grid.gen_index(phot.ray().pos()) {
        data.emission[index] += phot.power() * phot.weight();
    } else {
        panic!("Photon was not emitted within the grid.");
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
    while let Some((index, voxel)) = input.grid.gen_index_voxel(phot.ray().pos()) {
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
        let voxel_dist = voxel
            .dist(phot.ray())
            .expect("Could not determine voxel distance.");
        let scat_dist = -(rng.gen::<f64>()).ln() / env.inter_coeff();
        let surf_hit = input
            .tree
            .scan(phot.ray().clone(), bump_dist, voxel_dist.min(scat_dist));

        // Event handling.
        match Event::new(voxel_dist, scat_dist, surf_hit, bump_dist) {
            Event::Voxel(dist) => travel(&mut data, &mut phot, &env, index, dist + bump_dist),
            Event::Scattering(dist) => {
                travel(&mut data, &mut phot, &env, index, dist);

                // // Capture.
                // if let Some(weight) =
                //     crate::sim::mcrt::peel_off::peel_off(input, phot.clone(), &env, *detector_pos)
                // {
                //     detected_weight += weight;
                // }

                shift_scatter(&mut rng, &mut phot, &env);
            }
            Event::Surface(hit) => {
                travel(&mut data, &mut phot, &env, index, hit.dist());
                surface(&mut rng, &hit, &mut phot, &mut env, &mut data);
                travel(&mut data, &mut phot, &env, index, bump_dist);
            }
        }

        if phot.weight() <= 0.0 {
            break;
        }
    }
}

// //! Raman specialised photon-lifetime engine function.

// use super::super::{peel_off, simple_peel_off, Attributes, Data, Event, Local, Photon, Sample, Universe};
// use crate::{geom::Trace, math::sample_henyey_greenstein, phys::Crossing};
// use physical_constants::SPEED_OF_LIGHT_IN_VACUUM;
// use rand::{rngs::ThreadRng, Rng};
// use std::f64::consts::PI;

// /// Simulate the life of a single photon.
// /// Peel-off Raman detection.
// #[allow(clippy::expect_used)]
// #[inline]
// #[must_use]
// pub fn sample(rng: &mut ThreadRng, uni: &Universe, data: &mut Data, mut phot: Photon) -> Sample {
//     // Check photon is within the grid.
//     if let Some(index) = uni.grid.gen_index(phot.ray().pos()) {
//         data.emission_power[index] += phot.power() * phot.weight();
//     } else {
//         panic!("Photon was not emitted within the grid.");
//     }

//     // Common constants.
//     let bump_dist = uni.sett.bump_dist();
//     let loop_limit = uni.sett.loop_limit();
//     let roulette_weight = uni.sett.roulette_weight();
//     let roulette_barrels = uni.sett.roulette_barrels() as f64;
//     let roulette_survive_prob = 1.0 / roulette_barrels;

//     // Initialisation.
//     let mat = &uni.mats.map()[uni.sett.init_mat()];
//     let mut env = mat.env(phot.wavelength());

//     // Main loop.
//     let mut loops = 0;

//     while let Some((index, voxel)) = uni.grid.gen_index_voxel(phot.ray().pos()) {
//         // Loop limit check.
//         if loops >= loop_limit {
//             println!("Warning! Terminating photon: loop limit reached.");
//             break;
//         }
//         loops += 1;

//         // Roulette.
//         if phot.weight() <= roulette_weight {
//             let r = rng.gen::<f64>();
//             if r > roulette_survive_prob {
//                 break;
//             }
//             *phot.weight_mut() *= roulette_barrels;
//         }

//         // Interaction distances.
//         let voxel_dist = voxel
//             .dist(phot.ray())
//             .expect("Could not determine voxel distance.");
//         let scat_dist = -(rng.gen::<f64>()).ln() / env.inter_coeff();
//         let surf_hit = uni
//             .tree
//             .observe(phot.ray().clone(), bump_dist, voxel_dist.min(scat_dist));

//         // Event handling.
//         match Event::new(voxel_dist, scat_dist, surf_hit, bump_dist) {
//             Event::Voxel(dist) => travel(data, index, &env, &mut phot, dist + bump_dist),
//             Event::Scattering(dist) => {
//                 scatter(rng, data, index, &env, &mut phot, dist);

//                 //if phot.wavelength() >= 850.0e-9 {

//                     //if let Some(weight) =

//                         //peel_off(&env, uni, phot.clone(), *uni.sett.detector_pos())

//                     //{
//                         //data.total_raman_weight += weight * phot.weight() ;
//                     //}
//                 //}
//             }
//             Event::Surface(hit) => {
//                 // Move to the collision point.
//                 travel(data, index, &env, &mut phot, hit.dist());

//                 if let Some(attr) = uni.attrs.map().get(*hit.tag()) {
//                     match *attr {
//                         Attributes::Spectrometer => {
//                             data.spec.collect_weight(phot.wavelength(), phot.weight()*phot.power());
//                         }
//                         Attributes::Mirror => {
//                             *phot.ray_mut().dir_mut() =
//                                 Crossing::calc_ref_dir(phot.ray().dir(), hit.side().norm());
//                         }
//                         Attributes::Refractive {
//                             ref inside,
//                             ref outside,
//                         } => {
//                             // Determine far side material.
//                             let next_mat = if hit.side().is_inside() {
//                                 outside
//                             } else {
//                                 inside
//                             };

//                             // Get the near, and far side refractive indices.
//                             let curr_ref = env.ref_index();
//                             let next_env = uni.mats.map()[next_mat].env(phot.wavelength());
//                             let next_ref = next_env.ref_index();

//                             // Calculate the crossing normal vectors.
//                             let crossing = Crossing::new(
//                                 phot.ray().dir(),
//                                 hit.side().norm(),
//                                 curr_ref,
//                                 next_ref,
//                             );

//                             // Determine if a reflection or transmission occurs.
//                             let r = rng.gen::<f64>();
//                             if r <= crossing.ref_prob() {
//                                 // Reflect.
//                                 *phot.ray_mut().dir_mut() = *crossing.ref_dir();
//                             } else {
//                                 // Refract.
//                                 *phot.ray_mut().dir_mut() =
//                                     crossing.trans_dir().expect("Invalid refraction.");
//                                 env = next_env;
//                             }
//                         }
//                     }
//                 } else {
//                     panic!("Unknown attribute tag: {}", hit.tag());
//                 }

//                 // Move slightly away from the surface.
//                 travel(data, index, &env, &mut phot, bump_dist);
//             }
//         }
//     }

//     Sample::new(0.0)
// }

// /// Move the photon forward and record the flight.
// #[inline]
// fn travel(data: &mut Data, index: [usize; 3], env: &Local, phot: &mut Photon, dist: f64) {
//     debug_assert!(dist > 0.0);

//     let weight_power_dist = phot.weight() * phot.power() * dist;
//     data.energy[index] += weight_power_dist * env.ref_index() / SPEED_OF_LIGHT_IN_VACUUM;
//     data.absorptions[index] += weight_power_dist * env.abs_coeff();
//     data.shifts[index] += weight_power_dist * env.shift_coeff();

//     phot.ray_mut().travel(dist);
// }

// /// Perform a photon scattering event.
// #[inline]
// fn scatter(
//     rng: &mut ThreadRng,
//     data: &mut Data,
//     index: [usize; 3],
//     env: &Local,
//     phot: &mut Photon,
//     dist: f64,
// ) {
//     // Move to the interaction point.
//     travel(data, index, env, phot, dist);

//     // Part of the weight is absorbed.
//     *phot.weight_mut() *= env.albedo();

//     // The remaining weight may be shifted in a Raman/fluorescence event.
//     let r = rng.gen::<f64>();
//     if r <= env.shift_prob() {
//         // Shift occurs.
//         // Fluorescence event removes photons from optical range of interest.
//         *phot.weight_mut() = 0.0;
//         return;
//     }

//     // The remaining weight is scattered.
//     let phi = sample_henyey_greenstein(rng, env.asym());
//     let theta = rng.gen_range(0.0, PI * 2.0);
//     phot.ray_mut().rotate(phi, theta);
// }
