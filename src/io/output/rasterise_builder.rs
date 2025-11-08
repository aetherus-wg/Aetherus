use std::fmt::{Display, Formatter};

use arctk_attr::file;

use crate::{fmt_report, phys::synphot::TransmissionBuilder};

use super::Rasteriser;

#[file]
#[derive(Clone)]
pub enum RasteriseBuilder {
    Illuminance(TransmissionBuilder),
    PhotonCount,
}

impl RasteriseBuilder {
    pub fn build(&self) -> Rasteriser {
        match self {
            Self::Illuminance(ref tb) => Rasteriser::Illuminance(tb.build()),
            Self::PhotonCount => Rasteriser::PhotonCount,
        }
    }
}

impl Display for RasteriseBuilder {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Illuminance(ref tb) => {
                writeln!(fmt, "Illuminance: ...")?;
                fmt_report!(fmt, tb, "transmission")
            },
            Self::PhotonCount => {
                writeln!(fmt, "Count: ...")?;
            }
        }
        Ok(())
    }
}