//! Physics module.

pub mod crossing;
pub mod light;
pub mod local;
pub mod material;
pub mod photon;
pub mod reflectance;
pub mod spectrum;
pub mod synphot;

// Builders
pub mod light_linker_builder;
pub mod material_builder;
pub mod reflectance_builder;
pub mod spectrum_builder;

// Linkers
pub mod light_linker;

// Loaders
pub mod light_linker_builder_loader;

pub use self::{
    crossing::*, light::*, light_linker::*, light_linker_builder::*,
    light_linker_builder_loader::*, local::*, material::*, material_builder::*, photon::*,
    reflectance::*, reflectance_builder::*, spectrum::*, spectrum_builder::*,
};
