use arctk_attr::file;

use crate::phys::synphot::TransmissionBuilder;

use super::Rasteriser;

#[file]
pub enum RasteriseBuilder {
    Illuminance(TransmissionBuilder),
    CorrelatedColourTemperature,
}

impl RasteriseBuilder {
    pub fn build(&self) -> Rasteriser {
        match self {
            Self::Illuminance(ref tb) => Rasteriser::Illuminance(tb.build()),
            Self::CorrelatedColourTemperature => todo!(),
        }
    }
}