use assert_approx_eq::assert_approx_eq;
use rand::Rng;
use std::f64::consts::FRAC_PI_4;
use Aetherus as aether;

#[test]
fn pi_estimator() {
    let samples = 1e6 as i32;
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
