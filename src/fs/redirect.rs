//! File Redirection Struct
//!
//! Enables for file redirection in serialised and deserialised types.
//! This enables us to employ the 'builder' pattern for constructing deserialised objects.
//! This means that we can include either the type to be deserialised, or a the path to a file
//! that contains the type to be deserialised in its place. This struct will then
//! load the file in the appropriate place, using the appropriate loader.
//!
//! An example implementation of this pattern, using redirect can be seen below:
//! ```ignore
//! # use Aetherus::fs::{from_json_str, Redirect, Load};
//! # use Aetherus::err::Error;
//! # use tempfile::NamedTempFile;
//! # use std::io::Write;
//! # use serde_derive::{Deserialize};
//! # use std::path::Path;
//! # use arctk_attr::file;
//! # let mut redirect_file = NamedTempFile::new().unwrap();
//! // Create a structure that we are going to deserialise.
//! // Note that we have redirected fields here: we will directly load one
//! // and redirect the other to a temporary file.
//! #[file]
//! struct TestStructBuilder {
//!     struct1: Redirect<NestedStruct>,
//!     struct2 : Redirect<NestedStruct>,
//! }
//!
//! struct TestStruct {
//!     pub struct1: NestedStruct,
//!     pub struct2: NestedStruct,
//! }
//!
//! #[file]
//! struct NestedStruct {
//!     pub val1: f32,
//!     pub val2 : f32,
//! }
//!
//! impl Load for TestStructBuilder {
//!     type Inst = TestStruct;
//!
//!     #[inline]
//!     fn load(self, in_dir: &Path) -> Result<Self::Inst, Error> {
//!         let struct1 = self.struct1.load(in_dir)?;
//!         let struct2 = self.struct2.load(in_dir)?;
//!
//!         Ok(Self::Inst { struct1, struct2 })
//!     }
//! }
//!
//!// Create the JSON string to deserialise, making sure to include the
//!// Here and There variants where required.
//! let file_contents = format!("{{
//!     struct1 : {{ Here: {{ val1: 1.0, val2: 2.0 }} }},
//!     struct2 : {{ There: \"{}\" }}
//! }}", redirect_file.path().display());
//! redirect_file.reopen().unwrap();
//! redirect_file.write_all(b"{ val1: 4.0, val2: 5.0 }").unwrap();
//!
//! // Now we can directly deserialise these objects.
//! let deserialised = from_json_str::<TestStructBuilder>(&file_contents).unwrap();
//! let out = deserialised.load(Path::new("/")).unwrap();
//!
//! ```

use crate::{
    err::Error,
    fs::{as_json, from_json, File, Load, Save},
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter},
    path::Path,
};

/// Possible file redirection structure.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Redirect<T> {
    /// Path to file.
    There(String),
    /// Direct value.
    Here(T),
}

impl<T: File> File for Redirect<T>
where
    for<'de> T: Deserialize<'de>,
{
    #[inline]
    fn load(path: &Path) -> Result<Self, Error> {
        from_json(path)
    }
}

impl<T: Serialize> Save for Redirect<T> {
    #[inline]
    fn save_data(&self, path: &Path) -> Result<(), Error> {
        as_json(self, path)
    }
}

impl<T: File> Load for Redirect<T> {
    type Inst = T;

    #[inline]
    fn load(self, in_dir: &Path) -> Result<Self::Inst, Error> {
        match self {
            Self::There(path) => {
                let path = in_dir.join(path);
                T::new_from_file(&path)
            }
            Self::Here(val) => Ok(val),
        }
    }
}

impl<T: Display> Display for Redirect<T> {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        match *self {
            Self::There(ref path) => write!(fmt, "-> {}", path),
            Self::Here(ref item) => write!(fmt, "_! {}", item),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_redirect() {
        use super::{super::from_json_str, Load, Redirect};
        use crate::err::Error;
        use arctk_attr::file;
        use std::io::Write;
        use std::path::Path;
        use tempfile::NamedTempFile;

        let mut redirect_file = NamedTempFile::new().unwrap();
        // Create a structure that we are going to deserialise.
        // Note that we have redirected fields here: we will directly load one
        // and redirect the other to a temporary file.
        #[file]
        struct TestStructBuilder {
            struct1: Redirect<NestedStruct>,
            struct2: Redirect<NestedStruct>,
        }

        struct TestStruct {
            pub struct1: NestedStruct,
            pub struct2: NestedStruct,
        }

        #[file]
        struct NestedStruct {
            pub val1: f32,
            pub val2: f32,
        }

        impl Load for TestStructBuilder {
            type Inst = TestStruct;

            #[inline]
            fn load(self, in_dir: &Path) -> Result<Self::Inst, Error> {
                let struct1 = self.struct1.load(in_dir)?;
                let struct2 = self.struct2.load(in_dir)?;

                Ok(Self::Inst { struct1, struct2 })
            }
        }

        // Create the JSON string to deserialise, making sure to include the
        // Here and There variants where required.
        let file_contents = format!(
            "{{
            struct1 : {{ Here: {{ val1: 1.0, val2: 2.0 }} }},
            struct2 : {{ There: \"{}\" }}
        }}",
            redirect_file.path().display()
        );
        redirect_file.reopen().unwrap();
        redirect_file
            .write_all(b"{ val1: 4.0, val2: 5.0 }")
            .unwrap();

        // Now we can directly deserialise these objects.
        let deserialised = from_json_str::<TestStructBuilder>(&file_contents).unwrap();
        let out = deserialised.load(Path::new("/")).unwrap();

        // Check that we got the correct values back from the Here / There variants.
        assert_eq!(out.struct1.val1, 1.0);
        assert_eq!(out.struct1.val2, 2.0);
        assert_eq!(out.struct2.val1, 4.0);
        assert_eq!(out.struct2.val2, 5.0);
    }
}
