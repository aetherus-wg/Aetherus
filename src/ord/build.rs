//! Build trait.

use crate::err::Error;

/// Types implementing this trait can be built into another type.
pub trait Build {
    /// End type to be built.
    type Inst;

    /// Build the instance type.
    fn build(self) -> Result<Self::Inst, Error>;
}

impl<T: Build> Build for Vec<T> {
    type Inst = Vec<T::Inst>;
    fn build(self) -> Result<Self::Inst, Error> {
        let mut built = Vec::with_capacity(self.len());
        for item in self {
            built.push(item.build()?);
        }
        Ok(built)
    }
}
