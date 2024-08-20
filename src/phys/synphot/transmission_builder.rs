use crate::{
    math::{Probability, ProbabilityBuilder},
    phys::synphot::Transmission,
};
use std::fmt::{Display, Formatter};
use arctk_attr::file;

use super::vision::lumeff::LuminousEfficacyFunction;

#[file]
#[derive(Clone)]
pub enum TransmissionBuilder {
    Probability(ProbabilityBuilder),
    Photopic,
    Scotopic,
}

impl TransmissionBuilder {
    pub fn build(&self) -> Transmission {
        match &self {
            Self::Probability(ref prob_build) => {
                todo!()
            },
            Self::Photopic => LuminousEfficacyFunction::JuddVos.get(),
            Self::Scotopic => LuminousEfficacyFunction::ScotopicCIE1951.get(),
        }
    }
}

impl Display for TransmissionBuilder {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Probability(ref pb) => {
                todo!()
            },
            Self::Photopic => {
                writeln!(fmt, "Photopic")?;
            },
            Self::Scotopic => {
                writeln!(fmt, "Scotopic")?;
            },
        };
        Ok(())
    }
}