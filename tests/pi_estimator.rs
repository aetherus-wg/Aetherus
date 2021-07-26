use assert_approx_eq::assert_approx_eq;
use rand::Rng;
use Aetherus as aether;

#[test]
fn pi_estimator() {
    let samples = 1e6 as i32;

    let mut rng = rand::thread_rng();

    let mut a = aether::data::Average::new();
    for n in 0..samples {
        let x: f64 = rng.gen();
        let y: f64 = rng.gen();

        if (x.powi(2) + y.powi(2)) <= 1.0 {
            a += 1.0;
        } else {
            a += 0.0;
        }
    }

    assert_eq!(a.counts(), samples);
    assert_approx_eq!(a.ave(), 3.14159 / 4.0, 1.0e-3);
}
