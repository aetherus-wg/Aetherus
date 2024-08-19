use crate::{math::Probability, phys::synphot::Transmission};
use ndarray::Array1;

/**
 * Luminous Efficacy Functions. 
 * 
 * This module contains the photopic luminosity function (PLF) for human vision.
 * The PLF is a function of wavelength, and is defined in the range 360-830 nm.
 * The prescriptions that this function contains are:
 * - **vl1924.csv**: CIE (1924) photopic luminosity function. 
 * - **vlje.csv**: Judd (1951) modified CIE 2-deg photopic luminosity function. 
 * - **vljve.csv**: Judd-Vos (1978) modified CIE 2-deg photopic luminosity function. 
 */
const SCVLE: &str = include_str!("scvle.csv");
const VL1924E: &str = include_str!("vl1924e.csv");
const VLJE: &str = include_str!("vlje.csv");
const VLJVE: &str = include_str!("vljve.csv");

#[derive(Debug, Default)]
pub enum LuminousEfficacyFunction {
    PhotopicCIE1924,
    ScotopicCIE1951,
    Judd,
    #[default]
    JuddVos,
}

impl LuminousEfficacyFunction {
    pub fn parse_plf(&self, input: &str) -> Transmission {
        // Load all of the lines into a vector of lines.
        let lines: Vec<_> = input
            .lines()
            .filter(|line| !line.starts_with("//"))
            .collect();

        // As we know the number of rows, we can pre-allocate the rows vector.
        let mut lam = Vec::with_capacity(lines.len());
        let mut res = Vec::with_capacity(lines.len());
        // Now iterate the remaining lines, attempt to parse them and push them onto the rows vec.
        for line in lines {
            let row: Vec<f64> = line
                .split(',')
                .map(str::parse)
                .filter_map(Result::ok)
                .collect();
            lam.push(row[0]);
            res.push(row[1]);
        };

        let data_spline = Probability::new_linear_spline(&Array1::from(lam), &Array1::from(res));
        Transmission {
            spec: data_spline,
        }
    }

    pub fn get_input_string(&self) -> &str {
        match self {
            Self::PhotopicCIE1924 => VL1924E,
            Self::ScotopicCIE1951 => SCVLE,
            Self::Judd => VLJE,
            Self::JuddVos => VLJVE,
        }
    }

    pub fn get(&self) -> Transmission {
        self.parse_plf(self.get_input_string())
    }
}