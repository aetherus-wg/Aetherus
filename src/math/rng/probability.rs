//! Probability distribution implementation.

use crate::{
    err::Error,
    math::{distribution, Formula},
};
use ndarray::Array1;
use rand::Rng;
use std::{
    fmt::{Display, Formatter},
    fs::File,
    io::Write,
    result::Result,
};

/// Probability distribution formulae.
///
/// This enum provides easy sampling from and manipulation of probability distribution functions (PDFs).
/// The most important function that this object serves is performing random sampling from PDFs.
/// This enum supports a variety of different formulae:
/// - `Probability::Point`: A constant value.
/// - `Probability::Points`: Randomly sample one of a number of provided values.
/// - `Probability::Linear`: Sample from a single linear spline.
/// - `Probability::Uniform`: Uniform probability between two values.
/// - `Probability::Gaussian`: A Gaussian (normal) distribution.
/// - `Probability::ConstantSpline`: Sample from a CDF whose value is determined by a `Formula`.
/// - `Probability::LinearSpline`: Sample from a PDF where an arbitrary dataset is represented by (N - 1) linear splines.

#[derive(Clone, Debug, PartialEq)]
pub enum Probability {
    /// Point.
    Point {
        /// Constant value.
        c: f64,
    },
    /// Points.
    Points {
        /// Possible values.
        cs: Array1<f64>,
    },
    /// Linear.
    Linear {
        /// Gradient.
        grad: f64,
        /// Y-intercept.
        intercept: f64,
        /// Integration constant offset.
        offset: f64,
        /// Area beneath line in range.
        area: f64,
    },
    /// Uniform range.
    Uniform {
        /// Minimum value.
        min: f64,
        /// Maximum value.
        max: f64,
    },
    /// Gaussian distribution.
    Gaussian {
        /// Average value.
        mu: f64,
        /// Variance.
        sigma: f64,
    },
    /// Constant spline.
    ConstantSpline {
        /// Cumulative distribution function.
        cdf: Formula,
    },
    /// Linear spline.
    LinearSpline {
        /// Gradients.
        grads: Array1<f64>,
        /// Y-intercepts.
        intercepts: Array1<f64>,
        /// Integration constant offsets.
        offsets: Array1<f64>,
        /// Area beneath line in each range.
        areas: Array1<f64>,
        /// Cumulative distribution function.
        cdf: Array1<f64>,
        /// The values that correspond to the CDF.
        xs: Array1<f64>,
    },
}

impl Probability {
    /// Construct a new point instance.
    #[inline]
    #[must_use]
    pub const fn new_point(c: f64) -> Self {
        Self::Point { c }
    }

    /// Construct a new points instance.
    #[inline]
    #[must_use]
    pub fn new_points(cs: Array1<f64>) -> Self {
        debug_assert!(cs.len() > 1);
        Self::Points { cs }
    }

    /// Construct a new uniform instance.
    #[inline]
    #[must_use]
    pub fn new_uniform(min: f64, max: f64) -> Self {
        debug_assert!(min < max);
        Self::Uniform { min, max }
    }

    /// Construct a new linear instance.
    #[inline]
    #[must_use]
    pub fn new_linear([x0, x1]: [f64; 2], [p0, p1]: [f64; 2]) -> Self {
        let dx = x1 - x0;
        let dp = p1 - p0;

        let grad = dp / dx;
        let intercept = p0 - (grad * x0);
        let offset = if x0 < x1 {
            (0.5 * grad * x0).mul_add(x0, intercept * x0)
        } else {
            (0.5 * grad * x1).mul_add(x1, intercept * x1)
        };

        let area = 0.5 * (p0 + p1) * (x1 - x0);

        Self::Linear {
            grad,
            intercept,
            offset,
            area,
        }
    }

    /// Construct a new gaussian instance.
    #[inline]
    #[must_use]
    pub fn new_gaussian(mu: f64, sigma: f64) -> Self {
        debug_assert!(sigma > 0.0);
        Self::Gaussian { mu, sigma }
    }

    /// Construct a new constant spline instance.
    #[inline]
    #[must_use]
    pub fn new_constant_spline(xs: Array1<f64>, ps: &Array1<f64>) -> Self {
        debug_assert!(xs.len() > 1);
        debug_assert!(xs.len() == (ps.len() + 1));
        debug_assert!(ps.iter().all(|p| *p >= 0.0));

        let mut cdf = Vec::with_capacity(xs.len());
        let mut total = 0.0;
        cdf.push(total);
        for ((x_curr, x_next), prob) in xs.iter().zip(xs.iter().skip(1)).zip(ps.iter()) {
            let area = (x_next - x_curr) * prob;
            total += area;
            cdf.push(total);
        }
        let mut cdf = Array1::from(cdf);
        cdf /= total;

        Self::ConstantSpline {
            cdf: Formula::new_linear_spline_auto(cdf, xs),
        }
    }

    /// Construct a new linear spline instance.
    #[inline]
    #[must_use]
    pub fn new_linear_spline(xs: &Array1<f64>, ps: &Array1<f64>) -> Self {
        debug_assert!(xs.len() > 1);
        debug_assert!(xs.len() == ps.len());
        debug_assert!(ps.iter().all(|p| *p >= 0.0));

        let mut grads = Vec::with_capacity(xs.len() - 1);
        let mut intercepts = Vec::with_capacity(xs.len() - 1);
        let mut offsets = Vec::with_capacity(xs.len() - 1);
        let mut areas = Vec::with_capacity(xs.len() - 1);
        let mut cdf = Vec::with_capacity(xs.len());
        let mut total = 0.0;
        for ((x_curr, x_next), (p_curr, p_next)) in xs
            .iter()
            .zip(xs.iter().skip(1))
            .zip(ps.iter().zip(ps.iter().skip(1)))
        {
            let dx = *x_next - *x_curr;
            let dp = *p_next - *p_curr;

            let grad = dp / dx;
            let intercept = *p_curr - (grad * *x_curr);
            let offset = if *x_curr < *x_next {
                (0.5 * grad * *x_curr).mul_add(*x_curr, intercept * *x_curr)
            } else {
                (0.5 * grad * *x_next).mul_add(*x_next, intercept * *x_next)
            };
            let area = 0.5 * (*p_curr + *p_next) * (*x_next - *x_curr);

            grads.push(grad);
            intercepts.push(intercept);
            offsets.push(offset);
            areas.push(area);

            total += area;
            cdf.push(total);
        }
        let mut cdf = Array1::from(cdf);
        cdf /= total;

        Self::LinearSpline {
            grads: Array1::from(grads),
            intercepts: Array1::from(intercepts),
            offsets: Array1::from(offsets),
            areas: Array1::from(areas),
            cdf,
            xs: xs.clone(),
        }
    }

    /// Sample a number from the described distribution.
    #[inline]
    #[must_use]
    pub fn sample<R: Rng>(&self, rng: &mut R) -> f64 {
        match *self {
            Self::Point { ref c } => *c,
            Self::Points { ref cs } => cs[rng.gen_range(0..cs.len())],
            Self::Uniform { ref min, ref max } => rng.gen_range(*min..*max),
            Self::Linear {
                grad,
                intercept,
                offset,
                area,
            } => {
                let r = rng.gen_range(0.0..1.0_f64);
                ((2.0 * grad)
                    .mul_add(r.mul_add(area, offset), intercept * intercept)
                    .sqrt()
                    - intercept)
                    / grad
            }
            Self::Gaussian { ref mu, ref sigma } => distribution::sample_gaussian(rng, *mu, *sigma),
            Self::ConstantSpline { ref cdf } => cdf.y(rng.gen()),
            Self::LinearSpline {
                ref grads,
                ref intercepts,
                ref offsets,
                ref areas,
                ref cdf,
                xs: _,
            } => {
                let a = rng.gen_range(0.0..1.0);
                for (index, c) in cdf.iter().enumerate() {
                    if a < *c {
                        let grad = grads[index];
                        let intercept = intercepts[index];
                        let offset = offsets[index];
                        let area = areas[index];

                        let r = rng.gen_range(0.0..1.0_f64);
                        // Check to see if we have converged toward 0, then compensate by assuming zero gradient across the bin.
                        if grad.abs() > 1E-9 {
                            return ((2.0 * grad)
                                .mul_add(r.mul_add(area, offset), intercept * intercept)
                                .sqrt()
                                - intercept)
                                / grad;
                        } else {
                            return r.mul_add(area, offset) / intercept;
                        }
                    }
                }
                0.0
            }
        }
    }

    /// Whereas the `sample` method performs random draws to sample from the distribution contained in the object,
    /// this method returns the value of the CDF at a given cumulative probability, effectively performing a manual sampling of the CDF.
    /// The input
    #[inline]
    #[must_use]
    pub fn sample_at(&self, ps: f64) -> f64 {
        match *self {
            Self::Point { ref c } => *c,
            Self::Points { ref cs } => cs[ps as usize],
            Self::Uniform { ref min, ref max } => min + (max - min) * ps,
            Self::Linear {
                grad,
                intercept,
                offset,
                area,
            } => {
                debug_assert!(ps >= 0.0);
                debug_assert!(ps <= 1.0);

                let r = ps;
                ((2.0 * grad)
                    .mul_add(r.mul_add(area, offset), intercept * intercept)
                    .sqrt()
                    - intercept)
                    / grad
            }
            Self::Gaussian { mu: _, sigma: _ } => todo!(),
            Self::ConstantSpline { ref cdf } => cdf.y(ps),
            Self::LinearSpline {
                ref grads,
                ref intercepts,
                ref offsets,
                ref areas,
                ref cdf,
                xs: _,
            } => {
                debug_assert!(ps >= 0.0);
                debug_assert!(ps <= 1.0);

                let a = ps;
                for (index, c) in cdf.iter().enumerate() {
                    if a < *c {
                        let grad = grads[index];
                        let intercept = intercepts[index];
                        let offset = offsets[index];
                        let area = areas[index];

                        let bin_start = if index == 0 { 0.0 } else { cdf[index - 1] };
                        let bin_width = if index == 0 {
                            cdf[index]
                        } else {
                            cdf[index] - cdf[index - 1]
                        };
                        let r = (ps - bin_start) / (bin_width);
                        // Check to see if we have converged toward 0, then compensate by assuming zero gradient across the bin.
                        if grad.abs() > 1E-9 {
                            return ((2.0 * grad)
                                .mul_add(r.mul_add(area, offset), intercept * intercept)
                                .sqrt()
                                - intercept)
                                / grad;
                        } else {
                            return r.mul_add(area, offset) / intercept;
                        }
                    }
                }
                0.0
            }
        }
    }

    /// Outputs the PDF currently contained in this instance to a file at the provided path.
    ///
    /// **Note:** this is only implemented for `LinearSpline` variants.
    #[inline]
    #[must_use]
    pub fn pdf_to_file(&self, filename: &str) -> Result<(), Error> {
        let mut outfile = File::create(filename)?;

        match *self {
            Self::LinearSpline {
                grads: _,
                intercepts: _,
                offsets: _,
                areas: _,
                ref cdf,
                ref xs,
            } => {
                let mut prev = 0.0;
                for (index, cumulative_prob) in cdf.iter().enumerate() {
                    writeln!(outfile, "{}\t{}", xs[index], cumulative_prob - prev)?;
                    prev = *cumulative_prob;
                }
                Ok(())
            }
            _ => {
                unimplemented!()
            }
        }
    }

    /// Outputs the CDF currently contained in this instance to a file at the provided path.
    ///
    /// **Note:** this is only implemented for `LinearSpline` variants.
    #[inline]
    #[must_use]
    pub fn cdf_to_file(&self, filename: &str) -> Result<(), Error> {
        let mut outfile = File::create(filename)?;

        match *self {
            Self::LinearSpline {
                grads: _,
                intercepts: _,
                offsets: _,
                areas: _,
                ref cdf,
                ref xs,
            } => {
                for (index, cumulative_prob) in cdf.iter().enumerate() {
                    writeln!(outfile, "{}\t{}", xs[index], cumulative_prob)?;
                }
                Ok(())
            }
            _ => {
                unimplemented!()
            }
        }
    }
}

impl Display for Probability {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        let kind = match *self {
            Self::Point { .. } => "Point",
            Self::Points { .. } => "Points",
            Self::Uniform { .. } => "Uniform",
            Self::Linear { .. } => "Linear",
            Self::Gaussian { .. } => "Gaussian",
            Self::ConstantSpline { .. } => "Constant Spline",
            Self::LinearSpline { .. } => "Linear Spline",
        };
        write!(fmt, "{}", kind)
    }
}

#[cfg(test)]
mod tests {
    use super::Probability;
    use assert_approx_eq::assert_approx_eq;
    use ndarray::Array1;
    use std::f64::consts::PI;

    /// A unit test that implements the analytical test case of a PDF f(x) = cos(theta).
    /// In this case, the analytical solution is F(x) = sin(theta).
    /// If this test case passes it means that we are reproducing the CDF to the better
    /// than the 0.1% level.
    #[test]
    fn analytical_linear_spline_check() {
        // Construct our PDF using a typical cos() function between 0 -> PI / 2 rad.
        let xs: Vec<f64> = (0..90).map(|iang| (iang as f64) * (PI / 180.)).collect();
        let ps: Vec<f64> = xs.iter().map(|xs| xs.cos()).collect();
        let pdf = Probability::new_linear_spline(&Array1::from(xs), &Array1::from(ps));

        // Now perform the test - check that we do get sin(theta) back.
        let test_x: Vec<f64> = (0..100).map(|x| x as f64 / 100.0).collect();
        let _: Vec<()> = test_x
            .iter()
            .map(|x| {
                let sampl = pdf.sample_at(*x);
                // We are checking to around the 0.1% level.
                assert_approx_eq!(sampl, (x).asin(), (PI / 2_f64) / 1000.)
            })
            .collect();
    }
}
