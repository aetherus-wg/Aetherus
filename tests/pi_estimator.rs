use assert_approx_eq::assert_approx_eq;
use rand::Rng;
use std::f64::consts::FRAC_PI_4;
use Aetherus as aether;

/// This benchmark case is a barebones test of the core libraries.
/// This serves as a test for the averaging class, and our ability
/// to generate random numbers.
///
/// This is our implementation of the classic example of using the Monte Carlo
/// method to estimate $\pi$. By randomly sampling numbers from a uniform
/// distribution between 0.0 and 1.0, and testing which of those lie within a
/// circla centred at (0, 0), our average for the ratio of those that do and
/// those that do not will converge upon $\pi$ if we are successful in our
/// implementation.
#[test]
fn pi_estimator() {
    // The number of random samples we are going to take.
    let samples = 1e6 as i32;
    // The maximum difference allowable from pi for the test to pass.
    let max_delta = 0.001;

    let mut rng = rand::thread_rng();

    let mut a = aether::data::Average::new();
    for _ in 0..samples {
        let x: f64 = rng.gen();
        let y: f64 = rng.gen();

        if (x.powi(2) + y.powi(2)) <= 1.0 {
            a += 1.0;
        } else {
            a += 0.0;
        }
    }

    assert_eq!(a.counts(), samples);
    assert_approx_eq!(a.ave(), FRAC_PI_4, max_delta);
}
