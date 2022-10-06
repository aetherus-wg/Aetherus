use std::{
    path::Path, 
    fmt::Display,
};
use crate::{
    fmt_report,
    err::Error, 
    data::Table, 
    fs::File,
};


#[derive(Debug, Clone)]
pub enum Spectrum {
    Constant(f64),
    Tophat(f64, f64, f64),
    /// A collection of wavelengths (in nm), assumed to be sorted shortest to longest. 
    /// The independent value that is being represented as a function of wavelength. 
    Data(Vec<f64>, Vec<f64>),
}

impl Spectrum {
    /// Returns an instance which represents a uniform value between two wavelengths.
    pub fn new_constant(value: f64) -> Spectrum {
        Spectrum::Constant(value)
    }

    /// Returns an instance which represents a uniform value between two wavelengths.
    pub fn new_tophat(lower: f64, upper: f64, value: f64) -> Spectrum {
        Spectrum::Tophat(lower, upper, value)
    }

    /// Returns a new instance which represents a linear function between two wavelengths. 
    pub fn new_linear(lower: f64, upper: f64, lower_value: f64, upper_value: f64) -> Spectrum {
        Spectrum::Data(vec![lower, upper], vec![lower_value, upper_value])
    }

    /// Loads the wavelengths and independent values from a file, handling errors while it does it. 
    /// This function makes the assumption that wavelength (in nm) is the first column, and the 
    /// independent variable is the second column. 
    pub fn data_from_file(input_file: &Path) -> Result<Spectrum, Error> {
        let tab: Table<f64> = Table::load(input_file)?;
        let lams = tab.rows().iter().map(|r| r[0]).collect();
        let vals = tab.rows().iter().map(|r| r[1]).collect();
        let spec =  Spectrum::Data(lams, vals);
        Ok(spec)
    }

    /// Performs a linear interpolation of the loaded data to return a value. 
    pub fn value_at(&self, lam: f64) -> Option<f64> {

        match *self {
            Self::Constant(val) => Some(val),
            Self::Tophat(lower, upper, val) => {
                if lower <= lam && lam <= upper {
                    Some(val)
                } else {
                    None
                }
            },
            Self::Data(ref lams, ref vals) => {
                // First check that the wavelength falls within the region of wavelength spectrum that we cover.
                // If not, we are done. Just return a None. 
                if lams.iter().count() > 0 && (lam < *lams.iter().next().unwrap() || lam > *lams.iter().last().unwrap() ) {
                    return None
                }

                // First determine the index that is below
                match lams.iter().position(|t| lam >= *t) {
                    // Wavelength not within the array, so return None. 
                    None => None,
                    Some(idx) => {
                        if lams[idx] == lam {
                            // Exact match, so just return the value at the current index. 
                            Some(vals[idx])
                        } else {
                            // We need to interpolate. 
                            if idx == lams.iter().count() - 1 {
                                // This is the last item, so we can't interpolate.
                                None
                            } else {
                                let dval = vals[idx + 1 ] - vals[idx];
                                let sigma = (lam - lams[idx]) / (lams[idx + 1 ] - lams[idx]);
                                let terp = vals[idx] + dval * sigma;
                                Some(terp)
                            }
                        }
                    }
                }
            }
        }
    }
    
    pub fn min_lam(&self) -> Option<&f64> {
        match *self {
            Self::Constant(_) => None,
            Self::Tophat(ref lower, _, _) => Some(&lower),
            Self::Data(ref lams, _) => lams.iter().min_by(|a, b| a.total_cmp(b)),
        }
    }

    pub fn max_lam(&self) -> Option<&f64> {
        match *self {
            Self::Constant(_) => None,
            Self::Tophat(_, ref upper, _) => Some(&upper),
            Self::Data(ref lams, _) => lams.iter().max_by(|a, b| a.total_cmp(b)),
        }
    }

    pub fn min_val(&self) -> Option<&f64> {
        match *self {
            Self::Constant(ref value) => Some(&value),
            Self::Tophat(_, _, ref value) => Some(&value),
            Self::Data(_, ref vals) => vals.iter().min_by(|a, b| a.total_cmp(b)),
        }
    }

    pub fn max_val(&self) -> Option<&f64> {
        match *self {
            Self::Constant(ref value) => Some(&value),
            Self::Tophat(_, _, ref value) => Some(&value),
            Self::Data(_, ref vals) => vals.iter().max_by(|a, b| a.total_cmp(b)),
        }
    }


}

impl Display for Spectrum {
    #[inline]
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(fmt, "Spectrum: ")?;

        match *self {
            Self::Constant(ref value) => {
                writeln!(fmt, "Constant: ")?;
                fmt_report!(fmt, *value, "value");
                Ok(())
            },
            Self::Tophat(ref lower, ref upper, ref value) => {
                writeln!(fmt, "Tophat: ")?;
                fmt_report!(fmt, format!("{}..{}", lower, upper), "wavelength range");
                fmt_report!(fmt, value, "value");
                Ok(())
            }
            Self::Data(ref lam, _) => {
                writeln!(fmt, "Data: ")?;
                fmt_report!(fmt, lam.iter().count(), "no. points");
                if lam.iter().count() > 1 {
                    fmt_report!(fmt, format!("{}..{}", lam.iter().next().unwrap(), lam.iter().last().unwrap()), "wavelength range");
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::Spectrum;
    use tempfile::NamedTempFile;

    /// Test that the tophat spectrum produces the correct results.
    /// Also implicitly tests that the linear interpolation is working as expected.
    #[test]
    fn test_uniform_interp() {
        let spec = Spectrum::new_tophat(0.0, 1.0, 1.0);
        assert_eq!(spec.value_at(0.25), Some(1.0));
        assert_eq!(spec.value_at(0.5), Some(1.0));
        assert_eq!(spec.value_at(0.75), Some(1.0));
    }

    /// Test that the the linear function specturm produces the correct results. 
    /// Also implicitly tests that the linear interpolation is working as expected.
    #[test]
    fn test_linear_func_interp() {
        let spec = Spectrum::new_linear(0.0, 1.0, 0.0, 1.0);
        assert_eq!(spec.value_at(0.25), Some(0.25));
        assert_eq!(spec.value_at(0.5), Some(0.5));
        assert_eq!(spec.value_at(0.75), Some(0.75));
    }

    /// Tests that the loading of a simpple (linear) spectrum from a file works as expected. 
    #[test]
    fn test_linear_func_interp_from_file() {
        // Create a temporary file with the simple (2 point) linear spectrum in it. 
        let infile = NamedTempFile::new().expect("Expected Temporary file to write test spectrum");
        let mut file = infile.reopen().expect("Unable to open temp file to write test spectrum. ");
        file.write_all("lam, val\n0.0, 0.0\n1.0, 1.0\n".as_bytes()).expect("Unable to write test spectrum. ");

        // Now attempt to load in the spectrum we created, failing if the process fails. 
        let path = infile.path();
        let spec_res = Spectrum::data_from_file(&path);
        assert!(spec_res.is_ok());

        // Now sample from the loaded spectrum. 
        let spec = spec_res.unwrap();
        assert_eq!(spec.value_at(0.25), Some(0.25));
        assert_eq!(spec.value_at(0.5), Some(0.5));
        assert_eq!(spec.value_at(0.75), Some(0.75));
    }

    /// In this test I will ensure that the behaviour is as expected for samples that lie
    /// on and outside of the boundaries.
    #[test]
    fn test_spectrum_boundaries() {
        let spec = Spectrum::new_linear(0.0, 1.0, 0.0, 1.0);

        // First, let's ensure that if we sample outside the boundaries, we get a none. 
        assert!( spec.value_at(-0.1).is_none() );
        assert!( spec.value_at(1.1).is_none() );

        // Check that we retrieve the sample points on the first and last data points. 
        assert_eq!( spec.value_at(0.0), Some(0.0) );
        assert_eq!( spec.value_at(1.0), Some(1.0) );
    }
}