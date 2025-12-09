//! Photon scattering function.

use crate::{
    geom::Hit, img::Colour, io::output::{Output, OutputParameter}, math::Point3, ord::cartesian::{X, Y}, phys::{Crossing, Local, Photon}, sim::Attribute
};
use rand::{rngs::ThreadRng, Rng};

/// Handle a surface collision.
#[allow(clippy::expect_used)]
#[inline]
pub fn surface(
    rng: &mut ThreadRng,
    hit: &Hit<Attribute>,
    phot: &mut Photon,
    env: &mut Local,
    data: &mut Output,
) {
    match *hit.tag() {
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
            let r = rng.gen::<f64>();
            if r <= crossing.ref_prob() {
                // Reflect.
                *phot.ray_mut().dir_mut() = *crossing.ref_dir();
            } else {
                // Refract.
                *phot.ray_mut().dir_mut() = crossing.trans_dir().expect("Invalid refraction.");
                *env = next_env;
            }
        }
        Attribute::Mirror(abs) => {
            *phot.weight_mut() *= abs;
            *phot.ray_mut().dir_mut() = Crossing::calc_ref_dir(phot.ray().dir(), hit.side().norm());
        }
        Attribute::Spectrometer(id) => {
            data.specs[id].try_collect_weight(phot.wavelength(), phot.weight());
            phot.kill();
        }
        Attribute::Imager(id, width, ref orient) => {
            let projection = orient.pos() - phot.ray().pos();
            let x = ((orient.right().dot_vec(&projection) / width) + 1.0) / 2.0;
            let y = ((orient.up().dot_vec(&projection) / width) + 1.0) / 2.0;

            if (0.0..=1.0).contains(&x) && (0.0..=1.0).contains(&y) {
                let res = data.imgs[id].pixels().raw_dim();
                data.imgs[id].pixels_mut()
                    [[(res[X] as f64 * x) as usize, (res[Y] as f64 * y) as usize]] +=
                    wavelength_to_col(phot.wavelength()) * (phot.weight() * phot.power()) as f32;
            }

            phot.kill();
        }
        Attribute::Ccd(id, width, ref orient, ref binner) => {
            let projection = orient.pos() - phot.ray().pos();
            let x = ((orient.right().dot_vec(&projection) / width) + 1.0) / 2.0;
            let y = ((orient.up().dot_vec(&projection) / width) + 1.0) / 2.0;

            if (0.0..=1.0).contains(&x) && (0.0..=1.0).contains(&y) {
                let res = data.ccds[id].raw_dim();
                if let Some(bin) = binner.try_bin(phot.wavelength()) {
                    data.ccds[id][[
                        (res[X] as f64 * x) as usize,
                        (res[Y] as f64 * y) as usize,
                        bin,
                    ]] += phot.weight() * phot.power();
                }
            }

            phot.kill();
        }
        Attribute::Reflector(ref reflectance) => match reflectance.reflect(rng, &phot, hit) {
            Some(ray) => *phot.ray_mut() = ray,
            None => phot.kill(),
        },
        Attribute::PhotonCollector(id) => {
            if hit.side().is_inside() {
                return;
            }
            data.phot_cols[id].collect_photon(phot);
        },
        Attribute::AttributeChain(ref attrs) => {
            for attr in attrs.iter() {
                let hit_proxy = Hit::new(attr, hit.dist(), hit.side().clone());
                surface(rng, &hit_proxy, phot, env, data)
            }
        },
        Attribute::Rasterise(id, ref rasteriser) => {
            rasteriser.rasterise(rng, phot, &mut data.plane[id]);
        },
        Attribute::Hyperspectral(ref id, ref plane) => {
            assert_eq!(*data.vol[*id].param(), OutputParameter::Hyperspectral, "Hyperspectral output target not set to 'hyperspectral' param. ");

            let projected_xy = plane.project_onto_plane(phot.ray().pos());
            let hp_loc = Point3::new(projected_xy.0, projected_xy.1, phot.wavelength());
            let projected_area = plane.projected_pix_area(&data.vol[*id]);
            let spec_binsize = plane.hyperspectral_bin_size(&data.vol[*id]);
            match data.vol[*id].gen_index(&hp_loc) {
                Some(index) => data.vol[*id].data_mut()[index] += phot.power() * phot.weight() / (projected_area * spec_binsize),
                None => {},
            }
        }
    }
}

/// Determine the colour for a given wavelength.
#[inline]
#[must_use]
fn wavelength_to_col(wavelength: f64) -> Colour {
    debug_assert!(wavelength > 0.0);

    let gamma = 0.8;

    let (r, g, b) = if (380.0e-9..=440.0e-9).contains(&wavelength) {
        let attenuation = 0.7_f64.mul_add((wavelength - 380.0e-9) / (440.0e-9 - 380.0e-9), 0.3);
        (
            ((-(wavelength - 440.0e-9) / (440.0e-9 - 380.0e-9)) * attenuation).powf(gamma),
            0.0,
            attenuation.powf(gamma),
        )
    } else if (440.0e-9..=490.0e-9).contains(&wavelength) {
        (
            0.0,
            ((wavelength - 440.0e-9) / (490.0e-9 - 440.0e-9)).powf(gamma),
            1.0,
        )
    } else if (490.0e-9..=510.0e-9).contains(&wavelength) {
        (
            0.0,
            1.0,
            (-(wavelength - 510.0e-9) / (510.0e-9 - 490.0e-9)).powf(gamma),
        )
    } else if (510.0e-9..=580.0e-9).contains(&wavelength) {
        (
            ((wavelength - 510.0e-9) / (580.0e-9 - 510.0e-9)).powf(gamma),
            1.0,
            0.0,
        )
    } else if (580.0e-9..=645.0e-9).contains(&wavelength) {
        (
            1.0,
            (-(wavelength - 645.0e-9) / (645.0e-9 - 580.0e-9)).powf(gamma),
            0.0,
        )
    } else if (645.0e-9..=750.0e-9).contains(&wavelength) {
        let attenuation = 0.7_f64.mul_add((750.0e-9 - wavelength) / (750.0e-9 - 645.0e-9), 0.3);
        (attenuation.powf(gamma), 0.0, 0.0)
    } else {
        (0.0, 0.0, 0.0)
    };

    Colour::new(r as f32, g as f32, b as f32, 1.0)
}
