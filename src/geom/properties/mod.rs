//! Properties module.
//! 
//! Implementing these interfaces on each of the types that included in the simulation
//! allow them to interact with the simulation and the photon packets within it. 

pub mod collide;
pub mod emit;
pub mod trace;
pub mod transformable;

pub use self::{collide::*, emit::*, trace::*, transformable::*};
