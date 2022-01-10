//! Random number module.

pub mod distribution;
pub mod probability;

// Builders
pub mod probability_builder;

pub use self::{distribution::*, probability::*, probability_builder::*};
