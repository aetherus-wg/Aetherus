use crate::{
    fs::Save,
    data::Histogram, 
    img::Image, 
    err::Error,
    io::output::{OutputPlane, OutputRegistry, OutputVolume, PhotonCollector}, phys::Photon, 
};
use std::{
    ops::AddAssign,
    path::Path,
};
use ndarray::Array3;

use super::OutputParameter;

#[derive(Clone)]
pub struct Output {
    /// Output volumes.
    pub vol: Vec<OutputVolume>,
    /// Output planes. 
    pub plane: Vec<OutputPlane>,
    /// Photon Collectors. 
    pub phot_cols: Vec<PhotonCollector>,
    /// Spectra.
    pub specs: Vec<Histogram>,
    /// Image data. 
    pub imgs: Vec<Image>,
    /// CCD Data.
    pub ccds: Vec<Array3<f64>>,
    /// Photo data.
    pub photos: Vec<Image>,

    /// Contains the mapping between index and name for
    /// each of the output types. 
    pub reg: OutputRegistry,
}

impl Output {
    /// This function polls each of the output volumes in the output object to
    /// find the closest voxel distance based on the position of the current 
    /// photon packet. This will then return the shortest distance to the 
    /// the current voxel boundary. There may be a case where there is no voxel
    /// in the path of travel of the packet, in that case return `None`. 
    pub fn voxel_dist(&self, phot: &Photon) -> f64 {
        let dists: Vec<f64> = self.vol.iter()
            .map(|grid| { grid.voxel_dist(phot) })
            .filter(Option::is_some)
            .map(Option::unwrap)
            .collect();
        match dists.into_iter().reduce(f64::min) {
            Some(val) => val,
            None => f64::INFINITY,
        }
    }

    pub fn get_volumes_for_param(&self, param: OutputParameter) -> Vec<&OutputVolume> {
        self.vol.iter()
            .filter(|&vol| vol.param() == &param)
            .collect()
    }

    pub fn get_volumes_for_param_mut(&mut self, param: OutputParameter) -> Vec<&mut OutputVolume> {
        self.vol.iter_mut()
            .filter(|vol| vol.param() == &param)
            .collect()
    }
}

impl AddAssign for Output {
    #[inline]
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

        for (a, b) in self.imgs.iter_mut().zip(&rhs.imgs) {
            *a += b;
        }

        for (a, b) in self.ccds.iter_mut().zip(&rhs.ccds) {
            *a += b;
        }

        for (a, b) in self.photos.iter_mut().zip(&rhs.photos) {
            *a += b;
        }
    }
}


impl Save for Output {
    #[inline]
    fn save_data(&self, out_dir: &Path) -> Result<(), Error> {

        for (vol, name) in self.vol.iter().zip(self.reg.vol_reg.names_list()) {
            let path = out_dir.join(format!("volume_{}.nc", name.to_string()));
            vol.save(&path)?;
        }

        for (plane, name) in self.plane.iter().zip(self.reg.plane_reg.names_list()) {
            let path = out_dir.join(format!("plane_{}.nc", name.to_string()));
            plane.save(&path)?;
        }

        for (name, index) in self.reg.spec_reg.set().map().iter() {
            self.specs[*index].save(&out_dir.join(&format!("spectrometer_{}.csv", name)))?;
        }

        for (name, index) in self.reg.img_reg.set().map().iter() {
            self.imgs[*index].save(&out_dir.join(&format!("img_{}.png", name)))?;
        }

        for (name, index) in self.reg.ccd_reg.set().map().iter() {
            self.ccds[*index].save(&out_dir.join(&format!("ccd_{}.nc", name)))?;
        }

        for (n, photo) in self.photos.iter().enumerate() {
            photo.save(&out_dir.join(&format!("photo_{:03}.png", n)))?;
        }

        for (name, index) in self.reg.phot_cols_reg.set().map().iter() {
            self.phot_cols[*index]
                .save(&out_dir.join(&format!("photon_collector_{}.csv", name)))?;
        }

        Ok(())
    }
}
