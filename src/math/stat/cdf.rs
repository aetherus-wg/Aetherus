//! Cumulative Density Function Implementation.
use crate::{core::Real, err::Error};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{default::Default, fs::File, io::Write};
use splines::{Spline, Key, Interpolation};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CumulativeDistributionBin {
    pub cumulative_prob: Real,
    pub width: Real, 
    pub value: Real,
}

impl CumulativeDistributionBin {
    pub fn contains(&self, val: Real) -> bool {
        (val - self.cumulative_prob).abs() < self.width / 2.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CumulativeDistributionFunction {
    Bins(Vec<CumulativeDistributionBin>),
    Spline(Spline<Real, Real>),
}

impl CumulativeDistributionFunction {
    pub fn from_bins(cumulative_probability: Vec<Real>, values: Vec<Real>, bin_width: Real) -> Self {
        let bins = (0..cumulative_probability.iter().count())
            .map(|i| {
                CumulativeDistributionBin {
                    cumulative_prob: cumulative_probability[i],
                    width: bin_width,
                    value: values[i],
                }
            })
            .collect::<Vec<CumulativeDistributionBin>>();

        Self::Bins(bins)
    }

    pub fn from_spline_points(probs: Vec<Real>, values: Vec<Real>) -> Self {
        let keys = (0..probs.iter().count())
            .map(|i| {
                Key::new(probs[i], values[i], Interpolation::Linear)
            })
            .collect();

        Self::Spline(Spline::from_vec(keys))
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

        Self::from_spline_points(cumulative_prob, values)
    }

    /// Takes a random sample from the CDF.
    pub fn sample<R: Rng>(&self, rng: &mut R) -> Real {
        let rand_sample = rng.gen_range(0.0..1.0);

        match self {
            Self::Bins(ref bins) => {
                // We should iterate through the bin values until we find the first
                // that is lower than the random sample, as this is the bin that contains
                // the value we want to sample.
                let ibin = match bins
                    .iter()
                    .position(|bin| bin.contains(rand_sample))
                {
                    // This is the first bin edge that is greater than the search, so go to the previous one.
                    Some(ibin) => ibin - 1,
                    // If no bin edge is greater, then it must lie in the final bin.
                    None => bins.iter().count() - 1,
                };
                bins[ibin].value
            },
            Self::Spline(ref spline) => {
                // As we are using a splint interpolation, we can just sample the spline here.
                match spline.sample(rand_sample) {
                    Some(val) => val, 
                    None => spline.keys().last().unwrap().value,
                }
            }
        }
    }

    /// Writes the CDF to a file in tabular form.
    pub fn write_to_file(&self, filename: &str) -> Result<(), Error> {
        let mut outfile = File::create(filename)?;

        match self {
            Self::Bins(ref bins) => {
                // Write the header
                let _ = writeln!(outfile, "# cdf_bin_centre\tcdf_bin_width\tvalue");
                for i in 0..bins.iter().count() {
                    outfile.write_all(
                        format!(
                            "{}\t{}\t{}\n",
                            bins[i].cumulative_prob, bins[i].width, bins[i].value
                        )
                        .as_bytes(),
                    )?;
                }
            },
            Self::Spline(ref spline) => {
                // Write the header
                let _ = writeln!(outfile, "# cumulative_prob\tvalue");
                for i in 0..spline.keys().iter().count() {
                    outfile.write_all(
                        format!(
                            "{}\t{}\n",
                            spline.keys()[i].t, spline.keys()[i].value
                        )
                        .as_bytes(),
                    )?;
                }
            }
        }

        Ok(())
    }
}

impl Default for CumulativeDistributionFunction {
    fn default() -> Self {
        Self::Spline(Spline::from_vec(vec![]))
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
