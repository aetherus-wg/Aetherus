use crate::{fmt_report, phys::{synphot::Transmission, Photon}};
use rand::Rng;
use super::OutputPlane;
use std::fmt::{Display, Formatter};


#[derive(Debug, Clone, PartialEq)]
pub enum Rasteriser {
    /// Rasterises the illuminance of the photons fed to the rasteriser using 
    /// a provided transmission function. 
    Illuminance(Transmission),
    /// Counts the number of photon packets that trigger the rasteriser by summing
    /// their weights. 
    PhotonCount,
}

impl Rasteriser {
    pub fn rasterise<R: Rng>(&self, rng: &mut R, phot: &Photon, plane: &mut OutputPlane) {
        match self {
            Self::Illuminance(ref trans) => {
                let trans_prob = trans.sample(phot);
                let should_transmit = rng.gen_range(0.0..1.0) < trans_prob;

                if should_transmit {
                    let xy = plane.project(phot.ray().pos());
                    match plane.at_mut(xy.0, xy.1) {
                        Some(pix) => *pix += phot.weight() * phot.power(),
                        None => panic!("Illuminance rasterisation outside raster"),
                    } 
                };
            }
            Self::PhotonCount => {
                let xy = plane.project(phot.ray().pos());
                match plane.at_mut(xy.0, xy.1) {
                    Some(pix) => *pix += phot.weight(),
                    None => panic!("Photon count rasterisation outside raster"),
                }   
            }
        }
    }
}

impl Display for Rasteriser {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Illuminance(ref trans) => {
                writeln!(fmt, "Illuminance: ")?;
                fmt_report!(fmt, trans, "transmission function")
            }
            Self::PhotonCount => {
                writeln!(fmt, "PhotonCount: ...")?;
            }
        }
        Ok(())
    }
}