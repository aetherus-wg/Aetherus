//! Photon engine module
//! 
//! This module provides the controller for photons travelling through the simulation.
//! There are a number of pluggable engines provided in this module, however we also provide the tools for you to easily write your own. 

pub mod engine;
pub mod engine_builder;
pub mod engine_builder_loader;

// Provide our pre-built engines.
pub mod engines;

pub use self::{engine::*, engines::*, engine_builder::*, engine_builder_loader::*};