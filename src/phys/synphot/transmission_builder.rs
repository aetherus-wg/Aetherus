use crate::{
    math::{Probability, ProbabilityBuilder},
    phys::synphot::Transmission,
};
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