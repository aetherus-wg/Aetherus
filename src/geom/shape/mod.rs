//! Shape Primitives module.
//!
//! This module contains a number of geometric primitives that can be used within
//! the simulations, as well as generalised interfaces for representing triangular meshes.

pub mod cube;
pub mod mesh;
pub mod mesh_loader;
pub mod smooth_triangle;
pub mod track;
pub mod triangle;

pub use self::{cube::*, mesh::*, mesh_loader::*, smooth_triangle::*, track::*, triangle::*};
