//! Photon scattering function.

use crate::{
    geom::Hit,
    io::output::Output,
    phys::{Crossing, Local, Photon},
    sim::Attribute,
};
use rand::{Rng, RngExt};

/// Handle a surface collision.
#[allow(clippy::expect_used)]
pub fn surface<R: Rng>(
    rng: &mut R,
    hit: &Hit<Attribute>,
    phot: &mut Photon,
    env: &mut Local,
    data: &mut Output,
) {
    match hit.tag() {
        Attribute::Interface(inside, outside) => {
            // Reference materials.
            let (curr_mat, next_mat) = if hit.side().is_inside() {
                (inside, outside)
            } else {
                (outside, inside)
            };

            // Find local optical environments.
            let curr_env = curr_mat.sample_environment(phot.wavelength());
            let next_env = next_mat.sample_environment(phot.wavelength());

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
                phot.ray_mut().update_dir(*crossing.ref_dir());
            } else {
                // Refract.
                let new_dir = crossing.trans_dir().expect("Invalid refraction.");
                phot.ray_mut().update_dir(new_dir);
                *env = next_env;
            }
        }
        Attribute::Reflector(ref reflectance) => {
            match reflectance.reflect(rng, &phot, hit) {
                Some(ray) => *phot.ray_mut() = ray,
                None => phot.kill(),
            }
        }
        Attribute::Mirror(abs) => {
            *phot.weight_mut() *= abs;
            let new_dir = Crossing::calc_ref_dir(phot.ray().dir(), hit.side().norm());
            phot.ray_mut().update_dir(new_dir);
        }
        Attribute::Detector(id) => {
            if !hit.side().is_inside() {
                // FIXME: The photon collection happens before the new Uid is updated in the
                // photon, hence this is a very ugly walk-around to fix this issues which needs to be
                // sorted out properly.
                let mut future_phot = phot.clone();
                data.collect_photon(&mut future_phot, *id);
                if future_phot.weight() <= 0.0 {
                    phot.kill();
                }
            }
        }
        Attribute::AttributeChain(ref attrs) => {
            // FIXME: For some reason this is not working
            for attr in attrs {
                let hit_proxy = Hit::new(attr, hit.dist(), hit.side().clone());
                surface(rng, &hit_proxy, phot, env, data);
            }
        }
    }
}

