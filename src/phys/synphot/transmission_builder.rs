use crate::math::ProbabilityBuilder;
use arctk_attr::file;

#[file]
#[derive(Clone)]
pub enum TransmissionBuilder {
    Probability(ProbabilityBuilder),

}