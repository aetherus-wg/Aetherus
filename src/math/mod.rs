//! Mathematics module.

pub mod alias;
pub mod func;
pub mod linalg;
pub mod rng;
pub mod slice;
pub mod trans3_builder;

pub use self::{alias::*, func::*, linalg::*, rng::*, slice::*, trans3_builder::*};
