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

    #[test]
    fn test_uniform_interp() {
        let spec = Spectrum::new_uniform(0.0, 1.0, 1.0);
        assert_eq!(spec.interp(0.5), Some(1.0));
    }

    #[test]
    fn test_linear_func_interp() {
        let spec = Spectrum::new_linear(0.0, 1.0, 0.0, 1.0);
        assert_eq!(spec.interp(0.5), Some(0.5));
    }

    #[test]
    fn test_linear_func_interp_from_file() {
        let infile = NamedTempFile::new().expect("Expected Temporary file to write test spectrum");
        let mut file = infile.reopen().expect("Unable to open temp file to write test spectrum. ");
        file.write_all("lam, val\n0.0, 0.0\n1.0, 1.0\n".as_bytes()).expect("Unable to write test spectrum. ");

        let path = infile.path();
        let spec_res = Spectrum::from_file(&path);
        assert!(spec_res.is_ok());

        let spec = spec_res.unwrap();
        println!("{:?}", spec.val);
        assert_eq!(spec.interp(0.5), Some(0.5));
    }
}