//! Mathematics module.

pub mod alias;
pub mod func;
pub mod linalg;
pub mod rng;
pub mod slice;
pub mod stat;
pub mod trans3_builder;

pub use self::{alias::*, func::*, linalg::*, rng::*, slice::*, stat::*, trans3_builder::*};
