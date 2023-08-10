//! Aetherus File I/O module.
//!
//! In this module we have implemented:
//! - Foundational traits to enabled file load / save / redirect operations.

/// Include the foundational traits and types.
pub mod file;
pub mod load;
pub mod save;
pub use self::{file::*, load::*, save::*};

/// File redirection type.
pub mod redirect;
pub use redirect::Redirect;
/// Extensions for the implementation of different file types.
pub mod extensions;
pub use self::extensions::json::*;
pub use self::extensions::wavefront::*;
