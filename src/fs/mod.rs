//! Aetherus File I/O module.
//! 
//! In this module we have implemented:
//! - Foundational traits to enabled file load / save / redirect operations.

/// Include the foundational traits and types.
pub mod file;
pub mod load;
pub mod save;

/// Extensions for the implementation of different file types. 
pub mod extensions;