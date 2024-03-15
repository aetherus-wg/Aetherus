//! Name type.

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};

/// Human-readable identifier type.
#[derive(Debug, PartialEq, Clone, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct Name(String);

impl Name {
    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }

    /// Get the name as a string.
    #[inline]
    #[must_use]
    pub fn as_string(&self) -> String {
        self.0.clone()
    }
}

impl Display for Name {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{{{}}}", self.0)
    }
}
