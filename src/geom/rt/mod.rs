//! Ray Tracing Module
//! 
//! This module is part of the hit-scan system. 
//! This modulke is used in the photon engine to decide upon the course of action
//! follow when the photon is interacting with the environment. By tracing the
//! ray of the photon forward through the grid, we can check for:
//! - The transition into another voxel.
//! - A scattering event happening within a voxel. 
//! - Hitting a surface and transitioning to a new material. 
//! 

pub mod hit;
pub mod orient;
pub mod ray;
pub mod scan;
pub mod side;

pub use self::{hit::*, orient::*, ray::*, scan::*, side::*};
