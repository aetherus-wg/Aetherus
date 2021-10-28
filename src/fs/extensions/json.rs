//! JSON
//! 
//! JSON file handling through the [`serde`](https://crates.io/crates/serde) crate. 
//! Note that this supports JSON, as well as the JSON5 dialect, through the [`json5`](https://crates.io/crates/json5) crate. 

use crate::err::Error;
use serde::Deserialize;
use std::{fs::read_to_string, path::Path};

/// Deserialise the type in json format.
/// # Errors
/// if file can not be opened or read string can not be serialised into an instance of the required type.
#[inline]
pub fn from_json<T>(path: &Path) -> Result<T, Error>
where
    for<'de> T: Deserialize<'de>,
{
    let s = read_to_string(path)?;
    Ok(json5::from_str(&s)?)
}

/// Deserialise the type in json format.
/// # Errors
/// if string can not be serialised into an instance of the required type.
#[inline]
pub fn from_json_str<T>(s: &str) -> Result<T, Error>
where
    for<'de> T: Deserialize<'de>,
{
    Ok(json5::from_str(s)?)
}

#[cfg(test)]
mod test {
    use std::io::Write;
    use tempfile::NamedTempFile;
    use serde_derive::{Deserialize};
    use super::{from_json, from_json_str};

    const JSON_STR: &str = "{ \"num_prop\": 42, \"string_prop\": \"Lorem Ipsum\", \"boolean_prop\": true }";
    #[derive(Deserialize, PartialEq, Eq, Debug)]
    struct TestStruct {
        num_prop: i8,
        string_prop: String,
        boolean_prop: bool,
    }

    ///Testing to see whether the JSON deserialization works in the case that we
    /// load it from a string. 
    #[test]
    fn test_string_deserialisation() {
        let deserialised_object: TestStruct = from_json_str(JSON_STR).unwrap();
        assert_eq!(deserialised_object, TestStruct{ num_prop: 42, string_prop: "Lorem Ipsum".to_string(), boolean_prop: true });
    }

    /// Testing to see whether the JSON deserialisation works in the case that we
    /// load it from a file.
    #[test]
    fn test_file_deserialisation() {
        // First write to the temp file.
        let mut input_file = NamedTempFile::new().unwrap();

        input_file.reopen().unwrap();
        let _ = input_file.write_all(JSON_STR.as_bytes()).unwrap();
        
        // Now we have written the file, let's load the JSON from the file.
        let deserialised_object: TestStruct = from_json(input_file.path()).unwrap();

        assert_eq!(deserialised_object, TestStruct{ num_prop: 42, string_prop: "Lorem Ipsum".to_string(), boolean_prop: true });
    }
}