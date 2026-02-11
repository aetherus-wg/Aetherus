//! Distribution functions.

use crate::math::Dir3;
use lazy_static::lazy_static;
use rand::{Rng, RngExt};
use rand_distr::{Normal, Distribution};
use std::f64::consts::{FRAC_PI_2, PI};

lazy_static! {
    /// Golden ratio constant.
    static ref GOLDEN_RATIO: f64 = (1.0 + 5.0_f64.sqrt()) / 2.0;
}

/// Sample the Henyey-Greenstein phase function with a given asymmetry parameter.
#[must_use]
pub fn sample_henyey_greenstein<R: Rng>(rng: &mut R, asym: f64) -> f64 {
    debug_assert!(asym.abs() <= 1.0);

    if asym.abs() < 1.0e-6 {
        return rng.random_range(-1.0_f64..1.0).acos();
    }

    let asym_sq = asym * asym;

    let a = (1.0 - asym_sq) / asym.mul_add(rng.random_range(-1.0..1.0), 1.0);
    ((1.0 + asym_sq - (a * a)) / (2.0 * asym)).acos()
}

/// Sample the normal distribution.
#[must_use]
pub fn sample_normal<R: Rng>(rng: &mut R) -> f64 {
    let normal = Normal::new(0.0, 1.0).unwrap();
    normal.sample(rng)
}

/// Sample a gaussian distribution.
#[must_use]
pub fn sample_gaussian<R: Rng>(rng: &mut R, mu: f64, sigma: f64) -> f64 {
    debug_assert!(sigma > 0.0);
    let normal = Normal::new(mu, sigma).unwrap();
    normal.sample(rng)
}

/// Create a random unit vector.
#[must_use]
pub fn rand_isotropic_dir<R: Rng>(rng: &mut R) -> Dir3 {
    let theta = rng.random_range(0.0..(2.0 * PI));
    let z: f64 = rng.random_range(-1.0..1.0);

    let v = (1.0 - (z * z)).sqrt();

    let x = v * theta.cos();
    let y = v * theta.sin();

    Dir3::new(x, y, z)
}

// FIXME: How are the following getting random numbers since no Rng is used?

/// Sample points within a circle using the golden ratio.
#[must_use]
pub fn rand_circle_point(n: i32, max: i32) -> (f64, f64) {
    debug_assert!(n >= 0);
    debug_assert!(n < max);

    let r = f64::from(n) / f64::from(max - 1);
    let theta = f64::from(n) * *GOLDEN_RATIO;

    (r, theta)
}

/// Sample points on a sphere's surface using the golden ratio.
#[must_use]
pub fn rand_sphere_point(n: i32, max: i32) -> (f64, f64) {
    debug_assert!(n >= 0);
    debug_assert!(n < max);

    let d = f64::from(1 - max).mul_add(0.5, f64::from(n));
    let phi = ((2.0 * d) / f64::from(max)).asin() + FRAC_PI_2;
    let theta = ((2.0 * PI) / *GOLDEN_RATIO) * (d % *GOLDEN_RATIO);

    (phi, theta)
}

/// Sample points on a hemisphere's surface using the golden ratio.
#[must_use]
pub fn rand_hemisphere_point(n: i32, max: i32) -> (f64, f64) {
    debug_assert!(n >= 0);
    debug_assert!(n < max);

    rand_sphere_point(n, max * 2)
}
