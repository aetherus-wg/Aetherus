use crate::{err::Error, fmt_report, phys::Spectrum};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::Path};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SpectrumBuilder {
    Constant(f64),
    Spectrum(String),
    Tophat(f64, f64, f64),
    Linear(f64, f64, f64, f64),
}

impl SpectrumBuilder {
    pub fn build(&self) -> Result<Spectrum, Error> {
        match *self {
            Self::Constant(ref value) => Ok(Spectrum::new_constant(*value)),
            Self::Spectrum(ref input_file) => Spectrum::data_from_file(&Path::new(&input_file)),
            Self::Tophat(lower, upper, val) => Ok(Spectrum::new_tophat(lower, upper, val)),
            Self::Linear(lower, upper, lower_value, upper_value) => {
                Ok(Spectrum::new_linear(lower, upper, lower_value, upper_value))
            }
        }
    }
}

impl Display for SpectrumBuilder {
    #[inline]
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Constant(ref value) => {
                writeln!(fmt, "Constant: ")?;
                fmt_report!(fmt, value, "value");
                Ok(())
            }
            Self::Spectrum(ref input_file) => {
                writeln!(fmt, "Spectrum: ")?;
                fmt_report!(fmt, input_file, "input file");
                Ok(())
            }
            Self::Tophat(lower, upper, val) => {
                writeln!(fmt, "Uniform: ")?;
                fmt_report!(fmt, format!("{}..{}", lower, upper), "wavelength range");
                fmt_report!(fmt, val, "value");
                Ok(())
            }
            Self::Linear(lower, upper, lower_value, upper_value) => {
                writeln!(fmt, "Linear: ")?;
                fmt_report!(fmt, format!("{}..{}", lower, upper), "wavelength range");
                fmt_report!(
                    fmt,
                    format!("{}..{}", lower_value, upper_value),
                    "wavelength range"
                );
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{phys::Spectrum, sim::input};
    use tempfile::NamedTempFile;
    use std::io::{Write, Seek};
    use json5;

    #[test]
    fn test_constant_spectrum_builder() {
        let builder = SpectrumBuilder::Constant(1.0);
        let spectrum = builder.build().unwrap();
        assert_eq!(spectrum, Spectrum::new_constant(1.0));
    }

    #[test]
    fn test_spectrum_file_spectrum_builder() {
        // Create a temporary file with the simple (2 point) linear spectrum in it.
        let infile = NamedTempFile::new().expect("Expected Temporary file to write test spectrum");
        let mut file = infile
            .reopen()
            .expect("Unable to open temp file to write test spectrum. ");
        file.write_all("lam, val\n0.0, 0.0\n0.1, 0.1\n0.2, 0.2\n0.3, 0.3\n0.4, 0.4\n0.5, 0.5\n0.6, 0.6\n0.7, 0.7\n0.8, 0.8\n0.9, 0.9\n1.0, 1.0\n".as_bytes()).expect("Unable to write test spectrum. ");
        let tmp_path_str = infile.path().to_str().unwrap().to_string();

        let builder = SpectrumBuilder::Spectrum(tmp_path_str.clone());
        let spectrum = builder.build().unwrap();
        assert_eq!(spectrum, Spectrum::data_from_file(Path::new(&tmp_path_str)).unwrap());
    }

    #[test]
    fn test_tophat_spectrum_builder() {
        let builder = SpectrumBuilder::Tophat(400.0, 700.0, 1.0);
        let spectrum = builder.build().unwrap();
        assert_eq!(spectrum, Spectrum::new_tophat(400.0, 700.0, 1.0));
    }

    #[test]
    fn test_linear_spectrum_builder() {
        let builder = SpectrumBuilder::Linear(400.0, 700.0, 0.0, 1.0);
        let spectrum = builder.build().unwrap();
        assert_eq!(spectrum, Spectrum::new_linear(400.0, 700.0, 0.0, 1.0));
    }

    #[test]
    fn test_deserialize_constant_spectrum_builder() {
        let input_json = r#"{ Constant: 1.0 }"#;
        let infile = NamedTempFile::new().expect("Expected Temporary file to write test spectrum");
        let mut file = infile
            .reopen()
            .expect("Unable to open temp file to write test spectrum. ");
        file.write_all(input_json.as_bytes()).expect("Unable to write test spectrum. ");
        file.rewind().expect("Unable to rewind file. ");

        // Now read in using serde_json. 
        let json_str = std::fs::read_to_string(infile.path()).unwrap();
        let builder: SpectrumBuilder = json5::from_str(&json_str).unwrap();
        assert_eq!(builder, SpectrumBuilder::Constant(1.0));
    }

    #[test]
    fn test_deserialize_tophat_spectrum_builder() {
        let input_json = r#"{ Tophat: [0.0, 1.0, 1.0] }"#;
        let infile = NamedTempFile::new().expect("Expected Temporary file to write test spectrum");
        let mut file = infile
            .reopen()
            .expect("Unable to open temp file to write test spectrum. ");
        file.write_all(input_json.as_bytes()).expect("Unable to write test spectrum. ");
        file.rewind().expect("Unable to rewind file. ");

        // Now read in using serde_json. 
        let json_str = std::fs::read_to_string(infile.path()).unwrap();
        let builder: SpectrumBuilder = json5::from_str(&json_str).unwrap();
        assert_eq!(builder, SpectrumBuilder::Tophat(0.0, 1.0, 1.0));
    }

    #[test]
    fn test_deserialize_linear_spectrum_builder() {
        let input_json = r#"{ Linear: [0.0, 1.0, 0.0, 1.0] }"#;
        let infile = NamedTempFile::new().expect("Expected Temporary file to write test spectrum");
        let mut file = infile
            .reopen()
            .expect("Unable to open temp file to write test spectrum. ");
        file.write_all(input_json.as_bytes()).expect("Unable to write test spectrum. ");
        file.rewind().expect("Unable to rewind file. ");

        // Now read in using serde_json. 
        let json_str = std::fs::read_to_string(infile.path()).unwrap();
        let builder: SpectrumBuilder = json5::from_str(&json_str).unwrap();
        assert_eq!(builder, SpectrumBuilder::Linear(0.0, 1.0, 0.0, 1.0));
    }

    #[test]
    fn test_deserialize_spectrum_file_spectrum_builder() {
        let input_json = r#"{ Spectrum: "test_spectrum.csv" }"#;
        let infile = NamedTempFile::new().expect("Expected Temporary file to write test spectrum");
        let mut file = infile
            .reopen()
            .expect("Unable to open temp file to write test spectrum. ");
        file.write_all(input_json.as_bytes()).expect("Unable to write test spectrum. ");
        file.rewind().expect("Unable to rewind file. ");

        // Now read in using serde_json. 
        let json_str = std::fs::read_to_string(infile.path()).unwrap();
        let builder: SpectrumBuilder = json5::from_str(&json_str).unwrap();
        assert_eq!(builder, SpectrumBuilder::Spectrum("test_spectrum.csv".to_string()));
    }

    #[test]
    fn test_serialize_spectrum_builder() {
        let builder = SpectrumBuilder::Constant(1.0);
        let json_str = json5::to_string(&builder).unwrap();
        assert_eq!(json_str, r#"{"Constant":1}"#);

        let builder = SpectrumBuilder::Tophat(0.0, 1.0, 1.0);
        let json_str = json5::to_string(&builder).unwrap();
        assert_eq!(json_str, r#"{"Tophat":[0,1,1]}"#);

        let builder = SpectrumBuilder::Linear(0.0, 1.0, 0.0, 1.0);
        let json_str = json5::to_string(&builder).unwrap();
        assert_eq!(json_str, r#"{"Linear":[0,1,0,1]}"#);

        let builder = SpectrumBuilder::Spectrum("test_spectrum.csv".to_string());
        let json_str = json5::to_string(&builder).unwrap();
        assert_eq!(json_str, r#"{"Spectrum":"test_spectrum.csv"}"#);
    }
}