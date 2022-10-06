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


#[derive(Debug, Default, Clone)]
pub struct Spectrum {
    /// A collection of wavelengths (in nm), assumed to be sorted shortest to longest. 
    pub lam: Vec<f64>,
    /// The independent value that is being represented as a function of wavelength. 
    pub val: Vec<f64>,
}

impl Spectrum {
    /// Returns a new, empty instance of the Spectrum struct. 
    pub fn new() -> Spectrum {
        Spectrum { ..Default::default() }
    }

    /// Returns an instance which represents a uniform value between two wavelengths.
    pub fn new_uniform(lower: f64, upper: f64, value: f64) -> Spectrum {
        Spectrum { lam: vec![lower, upper], val: vec![value, value] }
    }

    /// Returns a new instance which represents a linear function between two wavelengths. 
    pub fn new_linear(lower: f64, upper: f64, lower_value: f64, upper_value: f64) -> Spectrum {
        Spectrum { lam: vec![lower, upper], val: vec![lower_value, upper_value] }
    }

    /// Loads the wavelengths and independent values from a file, handling errors while it does it. 
    /// This function makes the assumption that wavelength (in nm) is the first column, and the 
    /// independent variable is the second column. 
    pub fn load_file(&mut self, input_file: &Path) -> Result<(), Error> {
        let tab: Table<f64> = Table::load(input_file)?;

        let lams = tab.rows().iter().map(|r| r[0]).collect();
        let vals = tab.rows().iter().map(|r| r[1]).collect();

        self.lam = lams;
        self.val = vals;
        Ok(())
    }

    /// An initialiser which automatically tries to create a Spectrum object
    /// from a given input file. 
    pub fn from_file(input_file: &Path) -> Result<Spectrum, Error> {
        let mut spec = Spectrum::new();
        spec.load_file(input_file)?;
        Ok(spec)
    }

    /// Performs a linear interpolation of the loaded data to return a value. 
    pub fn interp(&self, lam: f64) -> Option<f64> {

        // First check that the wavelength falls within the region of wavelength spectrum that we cover.
        // If not, we are done. Just return a None. 
        if self.lam.iter().count() > 0 && (lam < *self.lam.iter().next().unwrap() || lam > *self.lam.iter().last().unwrap() ) {
            return None
        }

        // First determine the index that is below
        match self.lam.iter().position(|t| lam >= *t) {
            // Wavelength not within the array, so return None. 
            None => None,
            Some(idx) => {
                if self.lam[idx] == lam {
                    // Exact match, so just return the value at the current index. 
                    Some(self.val[idx])
                } else {
                    // We need to interpolate. 
                    if idx == self.lam.iter().count() - 1 {
                        // This is the last item, so we can't interpolate.
                        None
                    } else {
                        let dval = self.val[idx + 1 ] - self.val[idx];
                        let sigma = (lam - self.lam[idx]) / (self.lam[idx + 1 ] - self.lam[idx]);
                        let terp = self.val[idx] + dval * sigma;
                        Some(terp)
                    }
                }
            }
        }
    }
    
    pub fn min_lam(&self) -> Option<&f64> {
        self.lam.iter().min_by(|a, b| a.total_cmp(b))
    }

    pub fn max_lam(&self) -> Option<&f64> {
        self.lam.iter().max_by(|a, b| a.total_cmp(b))
    }

    pub fn min_val(&self) -> Option<&f64> {
        self.val.iter().min_by(|a, b| a.total_cmp(b))
    }

    pub fn max_val(&self) -> Option<&f64> {
        self.val.iter().max_by(|a, b| a.total_cmp(b))
    }
}

impl Display for Spectrum {
    #[inline]
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(fmt, "Spectrum: ")?;
        fmt_report!(fmt, self.lam.iter().count(), "no. points");
        if self.lam.iter().count() > 1 {
            fmt_report!(fmt, format!("{}..{}", self.lam.iter().next().unwrap(), self.lam.iter().last().unwrap()), "wavelength range");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::Spectrum;
    use tempfile::NamedTempFile;

    /// Test that the uniform spectrum produces the correct results.
    /// Also implicitly tests that the linear interpolation is working as expected.
    #[test]
    fn test_uniform_interp() {
        let spec = Spectrum::new_uniform(0.0, 1.0, 1.0);
        assert_eq!(spec.interp(0.25), Some(1.0));
        assert_eq!(spec.interp(0.5), Some(1.0));
        assert_eq!(spec.interp(0.75), Some(1.0));
    }

    /// Test that the the linear function specturm produces the correct results. 
    /// Also implicitly tests that the linear interpolation is working as expected.
    #[test]
    fn test_linear_func_interp() {
        let spec = Spectrum::new_linear(0.0, 1.0, 0.0, 1.0);
        assert_eq!(spec.interp(0.25), Some(0.25));
        assert_eq!(spec.interp(0.5), Some(0.5));
        assert_eq!(spec.interp(0.75), Some(0.75));
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
        let spec_res = Spectrum::from_file(&path);
        assert!(spec_res.is_ok());

        // Now sample from the loaded spectrum. 
        let spec = spec_res.unwrap();
        assert_eq!(spec.interp(0.25), Some(0.25));
        assert_eq!(spec.interp(0.5), Some(0.5));
        assert_eq!(spec.interp(0.75), Some(0.75));
    }

    /// In this test I will ensure that the behaviour is as expected for samples that lie
    /// on and outside of the boundaries.
    #[test]
    fn test_spectrum_boundaries() {
        let spec = Spectrum::new_linear(0.0, 1.0, 0.0, 1.0);

        // First, let's ensure that if we sample outside the boundaries, we get a none. 
        assert!( spec.interp(-0.1).is_none() );
        assert!( spec.interp(1.1).is_none() );

        // Check that we retrieve the sample points on the first and last data points. 
        assert_eq!( spec.interp(0.0), Some(0.0) );
        assert_eq!( spec.interp(1.0), Some(1.0) );
    }
}