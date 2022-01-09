//! Shape Primitives module. 
//! 
//! This module contains a number of geometric primitives that can be used within 
//! the simulations, as well as generalised interfaces for representing triangular meshes. 

pub mod cube;
pub mod triangle;
pub mod mesh;
pub mod smooth_triangle;

pub mod mesh_loader;

pub use self::{cube::*, triangle::*, mesh::*, smooth_triangle::*, mesh_loader::*};