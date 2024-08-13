use crate::{
    data::Histogram, 
    img::Image, 
    io::output::{OutputPlane, OutputVolume, OutputRegistry, PhotonCollector}, 
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