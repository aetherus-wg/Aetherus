//! Basis Labels for Enumerated Lists
//!
//! This module provides basic labels for the basis vectors for lists in a variety
//! of systems. This includes:
//! - `az`: Alphabest of numerals, used for consistency with mathematical equations.
//! - `col`: Colour systems
//! - `dim`: Physical coordinate systems.

pub mod az;
pub mod cols;
pub mod dim;

pub use self::{az::*, cols::*, dim::*};
