use std::{
    path::Path,
    fmt::Display,
};
use crate::{
    fmt_report,
    phys::Spectrum,
    err::Error
};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SpectrumBuilder {
    Spectrum(String),
    Tophat(f64, f64, f64),
    Linear(f64, f64, f64, f64),
}

impl SpectrumBuilder {
    pub fn build(&self) -> Result<Spectrum, Error> {
        match *self {
            Self::Spectrum(ref input_file) => Spectrum::data_from_file(&Path::new(&input_file)),
            Self::Tophat(lower, upper, val) => Ok(Spectrum::new_tophat(lower, upper, val)),
            Self::Linear(lower, upper, lower_value, upper_value) => Ok(Spectrum::new_linear(lower, upper, lower_value, upper_value)),
        }
    }
}

impl Display for SpectrumBuilder {
    #[inline]
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Spectrum(ref input_file) => {
                writeln!(fmt, "Spectrum: ")?;
                fmt_report!(fmt, input_file, "input file");
                Ok(())
            },
            Self::Tophat(lower, upper, val) => {
                writeln!(fmt, "Uniform: ")?;
                fmt_report!(fmt, format!("{}..{}", lower, upper), "wavelength range");
                fmt_report!(fmt, val, "value");
                Ok(())
            }, 
            Self::Linear(lower, upper, lower_value, upper_value) => {
                writeln!(fmt, "Linear: ")?;
                fmt_report!(fmt, format!("{}..{}", lower, upper), "wavelength range");
                fmt_report!(fmt, format!("{}..{}", lower_value, upper_value), "wavelength range");
                Ok(())
            },
        }
    }
}