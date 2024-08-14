use crate::{
    data::Histogram, 
    img::Image, 
    io::output::{OutputPlane, OutputRegistry, OutputVolume, PhotonCollector}, phys::Photon, 
};
use ndarray::Array3;

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
}