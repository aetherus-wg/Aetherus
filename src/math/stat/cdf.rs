//! Cumulative Density Function Implementation.
use crate::{
    core::Real,
    err::Error,
};
use rand::Rng;
use std::{fs::File, io::Write, default::Default};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CDFBin {
    pub step_cumulative_probability: Real, 
    pub value: Real, 
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum CumulativeDistributionFunction {
    Data(Vec<CDFBin>),
}

impl CumulativeDistributionFunction {
    pub fn from_cdf_data(cumulative_probability: Vec<Real>, values: Vec<Real>) -> Self {

        let bins: Vec<CDFBin> = (0..cumulative_probability.iter().count())
            .map(|i| {
                let edge = match i {
                    0 => 0.0, 
                    _ => (cumulative_probability[i] + cumulative_probability[i - 1]) / 2.0
                };

                CDFBin { step_cumulative_probability: edge, value: values[i] }
            })
            .collect();

        Self::Data(bins)
    }
    
    /// Takes in a correctly normalised probability density and converts to a cumulative density. 
    pub fn from_pdf(prob: Vec<Real>, values: Vec<Real>) -> Self {
        let mut accum: Real = 0.0;
        let mut cumulative_prob: Vec<Real> = prob
            .iter()
            .map(|prob| {
                let current_accum = accum.clone();
                accum += *prob;
                current_accum
            })
            .collect();

        // Now normalise the CDF so that the entire function occupies the correct range.
        cumulative_prob = cumulative_prob
            .into_iter()
            .map(|prob_unnorm| prob_unnorm / accum)
            .collect();

        Self::from_cdf_data(cumulative_prob, values)
    }

    /// Takes a random sample from the CDF.
    pub fn sample<R: Rng>(&self, rng: &mut R) -> Real {
        let rand_sample = rng.gen_range(0.0..1.0);

        match self {
            Self::Data(ref bins) => {
                // We should iterate through the bin values until we find the first
                // that is lower than the random sample, as this is the bin that contains
                // the value we want to sample. 
                let ibin = match bins.iter().position(|bin| bin.step_cumulative_probability > rand_sample) {
                    // This is the first bin edge that is greater than the search, so go to the previous one.
                    Some(ibin) => ibin - 1,
                    // If no bin edge is greater, then it must lie in the final bin. 
                    None => bins.iter().count() - 1,
                };
                bins[ibin].value
            }
        }
    }

    /// Writes the CDF to a file in tabular form.
    pub fn write_to_file(&self, filename: &str) -> Result<(), Error> {
        let mut outfile = File::create(filename)?;

        match self {
            Self::Data(ref bins) => {
                for i in 0..bins.iter().count() {
                    outfile.write_all(
                        format!("{}\t{}\n", bins[i].value, bins[i].step_cumulative_probability).as_bytes(),
                    )?;
                }
            }
        }

        Ok(())
    }
}

impl Default for CumulativeDistributionFunction {
    fn default() -> Self {
        Self::Data(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::CumulativeDistributionFunction;
    use crate::{core::Real, data::Average};
    use assert_approx_eq::assert_approx_eq;
    use statrs::distribution::{Continuous, Normal};
    use statrs::statistics::Distribution;

    #[test]
    fn test_gaussian_pdf_conversion() {
        let npts = 1000;
        let vals: Vec<Real> = (0..npts + 1)
            .into_iter()
            .map(|val| 2.0 * (val as Real) / (npts as Real))
            .collect();
        let norm = Normal::new(1.0, 0.5).unwrap();
        let mut probs: Vec<Real> = vals.iter().map(|val| norm.pdf(*val)).collect();

        let cdf = CumulativeDistributionFunction::from_pdf(probs, vals);

        let ntest = 10000;
        let mut rng = rand::thread_rng();
        let mut ave = Average::new();
        for _ in 0..ntest {
            ave += cdf.sample(&mut rng);
        }

        // Check that we get the value to within 1 percent. 
        assert_approx_eq!(ave.ave(), norm.mean().unwrap(), 1E-2);
    }
}
