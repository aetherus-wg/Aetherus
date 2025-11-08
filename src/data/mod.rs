//! Data organisation and reduction structures.

pub mod average;
pub mod histogram;
pub mod histogram_builder;
pub mod histogram_iter;
pub mod table;

pub use self::{average::*, histogram::*, histogram_builder::*, histogram_iter::*, table::*};
