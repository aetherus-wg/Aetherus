use crate::{
    data::Histogram,
    err::Error,
    fmt_report,
    fs::Save,
    geom::{Orient, properties::Trace},
    img::{Colour, Image},
    io::output::{AxisAlignedPlane, OutputPlane, OutputRegistry, OutputVolume, PhotonCollector, Rasteriser},
    math::Point3,
    ord::cartesian::{X, Y},
    phys::{Local, Photon},
    sim::travel,
    tools::Binner
};
use ndarray::Array3;
use std::{
    fmt::{Display, Formatter},
    ops::AddAssign,
    path::Path,
};

use physical_constants::SPEED_OF_LIGHT_IN_VACUUM;

use super::OutputParameter;

#[derive(Default, Debug, Clone)]
pub enum DetectorType {
    #[default]
    PhotonCollector,
    Spectrometer,
    Imager {
        width:  f64,
        height: f64,
        orient: Orient,
    },
    Ccd {
        width:  f64,
        height: f64,
        orient: Orient,
        binner: Binner,
    },
    Rasterise {
        rasteriser: Rasteriser,
    },
    Hyperspectral {
        plane: AxisAlignedPlane,
    },
}

#[derive(Debug, Clone)]
pub struct Detector {
    pub id: usize,
    pub det_type: DetectorType,
}

#[derive(Clone)]
pub struct Output {
    /// Output volumes.
    pub vol: Vec<OutputVolume>,
    /// Output planes.
    pub plane: Vec<OutputPlane>,
    /// Detectors.
    pub detectors: Vec<Detector>,
    /// Photon Collectors.
    pub phot_cols: Vec<PhotonCollector>,
    /// Spectral.
    pub specs: Vec<Histogram>,
    /// Image data.
    pub images: Vec<Image>,
    /// CCD Data.
    pub ccds: Vec<Array3<f64>>,

    /// Contains the mapping between index and name for
    /// each of the output types.
    pub reg: OutputRegistry,
}

fn voxels_march<F>(
    vol: &mut OutputVolume,
    env: &Local,
    phot: &Photon,
    mut dist: f64,
    bump_dist: f64,
    delta_fn: &F,
) where
    F: Fn(f64, f64) -> f64,
{
    assert!(
        dist > 0.0,
        "Photon travel distance must be positive non-zero"
    );

    let mut tmp_phot = phot.clone();

    // 1. First move tmp_phot to the voxel boundary
    if vol.contains(phot.ray().pos()) {
        // Photon is already inside the output volume
    } else {
        if let Some(boundary_dist) = vol.boundary_dist(&tmp_phot) {
            travel(&mut tmp_phot, &env, boundary_dist + bump_dist);
            dist -= boundary_dist + bump_dist;
        } else {
            // This output volume has not been found in the path of this photon
            return;
        }
    }

    // 2. Max travel to within the volume of interest
    let mut tmp_dist = dist.min(vol.boundary_dist(&tmp_phot).unwrap_or(0.0));

    // 3. Iterate throgh the voxels, until the distance is consumed
    while tmp_dist > 0.0 {
        let (index, voxel) = match vol.gen_index_voxel(tmp_phot.ray().pos()) {
            Some(inner) => inner,
            None => break,
        };
        let voxel_dist = match voxel.dist(tmp_phot.ray()) {
            Some(dist) => dist,
            None => break,
        };

        if voxel_dist == 0.0 {
            println!("Investigate voxel at index {:?}", index);
        }

        debug_assert!(voxel_dist >= 0.0, "Cannot travel backwards");
        debug_assert!(tmp_dist >= 0.0, "Cannot travel backwards");

        let mut step = voxel_dist.min(tmp_dist);
        debug_assert!(step > 0.0, "Step size must be positive non-zero");

        step += bump_dist;
        tmp_dist -= step;

        let voxel_in_power = tmp_phot.weight() * tmp_phot.power();

        // Compute the effective distance that results in the same energy accumulation for
        // a non absorbing medium.
        let effective_step = if env.abs_coeff() < f64::MIN_POSITIVE {
            step
        } else {
            (1.0 - (-env.abs_coeff() * step).exp()) / env.abs_coeff()
        };

        debug_assert!(effective_step > 0.0, "Step size must be positive non-zero");

        // Step temporal photon to the next voxel
        travel(&mut tmp_phot, &env, step);

        debug_assert!(tmp_phot.ray() != phot.ray());

        vol.data_mut()[index] += delta_fn(voxel_in_power, effective_step);
    }
}

impl Output {
    pub fn volume_estimate(&mut self, env: &Local, phot: &Photon, dist: f64, bump_dist: f64) {
        assert!(env.abs_coeff() >= 0.0);

        // Energy Density.
        let energy_fn = |voxel_in_power: f64, effective_step: f64| {
            voxel_in_power * effective_step * env.ref_index() / SPEED_OF_LIGHT_IN_VACUUM
        };
        self.get_volumes_for_param_mut(OutputParameter::Energy)
            .iter_mut()
            .for_each(|vol| {
                voxels_march(vol, &env, &phot, dist, bump_dist, &energy_fn);
            });

        // Absorption.
        let absorption_fn = |voxel_in_power: f64, effective_step: f64| {
            voxel_in_power * effective_step * env.abs_coeff() * env.ref_index()
                / SPEED_OF_LIGHT_IN_VACUUM
        };
        self.get_volumes_for_param_mut(OutputParameter::Absorption)
            .iter_mut()
            .for_each(|vol| {
                voxels_march(vol, env, phot, dist, bump_dist, &absorption_fn);
            });

        // Shifts.
        let shift_fn = |voxel_in_power: f64, effective_step: f64| {
            voxel_in_power * effective_step * env.shift_coeff() * env.ref_index()
                / SPEED_OF_LIGHT_IN_VACUUM
        };
        self.get_volumes_for_param_mut(OutputParameter::Shift)
            .iter_mut()
            .for_each(|vol| {
                voxels_march(vol, env, phot, dist, bump_dist, &shift_fn);
            });
    }

    pub fn collect_photon(&mut self, phot: &mut Photon, id: usize) {
        let inner_id = self.detectors[id].id;
        match &self.detectors[id].det_type {
            DetectorType::PhotonCollector => {
                self.phot_cols[inner_id].collect_photon(phot);
            }
            DetectorType::Spectrometer => {
                self.specs[inner_id].try_collect_weight(phot.wavelength(), phot.weight());
                phot.kill();
            }
            DetectorType::Imager {width, height, orient} => {
                let projection = orient.pos() - phot.ray().pos();
                let x = ((orient.right().dot_vec(&projection) / width) + 1.0) / 2.0;
                let y = ((orient.up().dot_vec(&projection) / height) + 1.0) / 2.0;

                if (0.0..=1.0).contains(&x) && (0.0..=1.0).contains(&y) {
                    let res = self.images[inner_id].pixels().raw_dim();
                    self.images[inner_id].pixels_mut()
                        [[(res[X] as f64 * x) as usize, (res[Y] as f64 * y) as usize]] +=
                        wavelength_to_col(phot.wavelength())
                            * (phot.weight() * phot.power()) as f32;
                }

                phot.kill();
            }
            DetectorType::Ccd {
                width,
                height,
                orient,
                binner,
            } => {
                let projection = orient.pos() - phot.ray().pos();
                let x = ((orient.right().dot_vec(&projection) / width) + 1.0) / 2.0;
                let y = ((orient.up().dot_vec(&projection) / height) + 1.0) / 2.0;

                if (0.0..=1.0).contains(&x) && (0.0..=1.0).contains(&y) {
                    let res = self.ccds[inner_id].raw_dim();
                    if let Some(bin) = binner.try_bin(phot.wavelength()) {
                        self.ccds[inner_id][[
                            (res[X] as f64 * x) as usize,
                            (res[Y] as f64 * y) as usize,
                            bin,
                        ]] += phot.weight() * phot.power();
                    }
                }

                phot.kill();
            }
            DetectorType::Rasterise { rasteriser } => {
                rasteriser.rasterise(phot, &mut self.plane[inner_id]);
            }
            DetectorType::Hyperspectral { plane } => {
                assert_eq!(
                    *self.vol[inner_id].param(),
                    OutputParameter::Hyperspectral,
                    "Hyperspectral output target not set to 'hyperspectral' param. "
                );

                // FIXME: Maybe use Array3 instead of Point3, to make obvious the difference to a point
                // in space and a (x, y, lambda) vector
                let projected_xy = plane.project_onto_plane(phot.ray().pos());
                let hp_loc = Point3::new(projected_xy.0, projected_xy.1, phot.wavelength());
                let projected_area = plane.projected_pix_area(&self.vol[inner_id]);
                let spec_binsize = plane.hyperspectral_bin_size(&self.vol[inner_id]);
                match self.vol[inner_id].gen_index(&hp_loc) {
                    Some(index) => {
                        self.vol[inner_id].data_mut()[index] +=
                            phot.power() * phot.weight() / (projected_area * spec_binsize)
                    }
                    None => {}
                }
            }
        }
    }

    /// This function polls each of the output volumes in the output object to
    /// find the closest voxel distance based on the position of the current
    /// photon packet. This will then return the shortest distance to the
    /// the current voxel boundary. There may be a case where there is no voxel
    /// in the path of travel of the packet, in that case return `None`.
    pub fn voxel_dist(&self, phot: &Photon) -> f64 {
        let dists: Vec<f64> = self.vol
            .iter()
            .filter_map(|grid| grid.voxel_dist(phot))
            .collect();
        dists.into_iter().reduce(f64::min).unwrap_or(f64::INFINITY)
    }

    pub fn get_volumes_for_param(&self, param: OutputParameter) -> Vec<&OutputVolume> {
        self.vol
            .iter()
            .filter(|&vol| vol.param() == &param)
            .collect()
    }

    pub fn get_volumes_for_param_mut(&mut self, param: OutputParameter) -> Vec<&mut OutputVolume> {
        self.vol
            .iter_mut()
            .filter(|vol| vol.param() == &param)
            .collect()
    }
}

impl AddAssign for Output {
    fn add_assign(&mut self, rhs: Self) {
        for (a, b) in self.vol.iter_mut().zip(&rhs.vol) {
            *a += b;
        }

        for (a, b) in self.plane.iter_mut().zip(&rhs.plane) {
            *a += b;
        }

        for (a, b) in self.phot_cols.iter_mut().zip(&rhs.phot_cols) {
            *a += b;
        }

        for (a, b) in self.specs.iter_mut().zip(&rhs.specs) {
            *a += b;
        }

        for (a, b) in self.images.iter_mut().zip(&rhs.images) {
            *a += b;
        }

        for (a, b) in self.ccds.iter_mut().zip(&rhs.ccds) {
            *a += b;
        }
    }
}

impl Save for Output {
    fn save_data(&self, out_dir: &Path) -> Result<(), Error> {
        for (vol, name) in self.vol.iter().zip(self.reg.vol_reg.names_list()) {
            let path = out_dir.join(format!("volume_{name}.nc"));
            vol.save(&path)?;
        }

        for (plane, name) in self.plane.iter().zip(self.reg.plane_reg.names_list()) {
            let path = out_dir.join(format!("plane_{name}.nc"));
            plane.save(&path)?;
        }

        for (name, index) in self.reg.spec_reg.set().map().iter() {
            self.specs[*index].save(&out_dir.join(format!("spectrometer_{name}.csv")))?;
        }

        for (name, index) in self.reg.images_reg.set().map().iter() {
            self.images[*index].save(&out_dir.join(format!("img_{name}.png")))?;
        }

        for (name, index) in self.reg.ccd_reg.set().map().iter() {
            self.ccds[*index].save(&out_dir.join(format!("ccd_{name}.nc")))?;
        }

        for (name, index) in self.reg.phot_cols_reg.set().map().iter() {
            self.phot_cols[*index]
                .save(&out_dir.join(format!("photon_collector_{name}.csv")))?;
        }

        Ok(())
    }
}

impl Display for Output {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        writeln!(fmt, "...")?;

        fmt_report!(fmt, self.reg.vol_reg, "output volume register");
        fmt_report!(fmt, self.reg.plane_reg, "output plane register");
        fmt_report!(fmt, self.reg.phot_cols_reg, "photon collector register");
        fmt_report!(fmt, self.reg.spec_reg, "spectrometer register");
        fmt_report!(fmt, self.reg.images_reg, "imager register");
        fmt_report!(fmt, self.reg.ccd_reg, "ccd register");

        fmt_report!(fmt, self.specs.len(), "spectrometers");
        fmt_report!(fmt, self.images.len(), "images");
        fmt_report!(fmt, self.ccds.len(), "ccds");

        fmt_report!(fmt, self.phot_cols.len(), "photon collectors");
        Ok(())
    }
}

// ------------------------------------
// Utils
// ------------------------------------

/// Determine the colour for a given wavelength.
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
