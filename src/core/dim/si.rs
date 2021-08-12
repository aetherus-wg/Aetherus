//! International System of Units

use crate::core::Real;

macro_rules! unit {
    ($underlying:ty, $name:ident, $value:expr, $dim:ident) => {
        pub const $name: $dim = dimensioned::si::SI {
            value_unsafe: $value,
            _marker: std::marker::PhantomData,
        };
    };
    ($underlying:ty, $name:ident, $value:expr, [$d0:ident, $d1:ident, $d2:ident, $d3:ident, $d4:ident, $d5:ident, $d6:ident]) => {
        use dimensioned::tarr;
        pub const $name: dimensioned::si::SI<
            $underlying,
            tarr![$d0, $d1, $d2, $d3, $d4, $d5, $d6],
        > = dimensioned::si::SI {
            value_unsafe: $value,
            _marker: std::marker::PhantomData,
        };
    };
}

// use dimensioned::typenum::{P4, Z0};
// unit!(Real, SPEED_OF_LIGHT, 29982293.0, Velocity);
// unit!(Real, XX, 1.0, [P4, Z0, Z0, Z0, Z0, Z0, Z0]);

// pub const SPEED_OF_LIGHT: Velocity = 29982293.0 * MPS;

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_newtons_second_law() {
//         let mass = 6.0 * KG;
//         let accel = 2.0 * MPS2;

//         let force = mass * accel;

//         assert_eq!(force, 12.0 * N);
//     }

//     #[test]
//     fn test_archimedes_principle() {
//         let fluid_density = 0.9 * KGPM3;
//         let grav_accel = 9.81 * MPS2;
//         let displacement_vol = 1.0 * M3;

//         let buoyant_force = -fluid_density * grav_accel * displacement_vol;

//         assert_eq!(buoyant_force, -8.829 * N);
//     }

//     #[test]
//     fn test_ohms_law() {
//         let voltage = 20.0 * V;
//         let resis = 5.0 * OHM;

//         let current = voltage / resis;

//         assert_eq!(current, 4.0 * A);
//     }

//     #[test]
//     fn test_coulombs_law() {
//         let charge_a = 0.1 * C;
//         let charge_b = 2.5 * C;
//         let distance = 10.0 * M;

//         let k = 8.9875e9 * N * M2 / C2; // TODO: Replace with constant.

//         let force = k * (charge_a * charge_b) / (distance * distance);

//         assert_eq!(force, 22468750.0 * N);
//     }

//     #[test]
//     fn test_stefans_law() {
//         let area = 2.0 * M2;
//         let temp = 2000.0 * K;

//         let sigma = 5.6703e-8 * W / (M * M * K * K * K * K); // TODO: Replace with constant.

//         let lumin = area * sigma * temp * temp * temp * temp;

//         assert_eq!(lumin, 1814496.0 * W);
//     }

//     #[test]
//     fn test_pascals_law() {
//         let force = 2000.0 * N;
//         let area = 2.0 * M2;

//         let pressure = force / area;

//         assert_eq!(pressure, 1000.0 * PA);
//     }

//     #[test]
//     fn test_hookes_law() {
//         let length = 0.1 * M;
//         let k = 2.0 * N / M;

//         assert_eq!(-k * length, -0.2 * N);
//     }

//     #[test]
//     fn test_bernoullis_law() {
//         let vel = 2.0 * MPS;
//         let pressure = 3.0 * PA;
//         let density = 0.1 * KGPM3;
//         let elevation = 1000.0 * M;

//         let g = 9.81 * MPS2; // TODO: Replace with constant.

//         assert_eq!(
//             ((vel * vel) / 2.0) + (pressure / density) + (g * elevation),
//             2.0 * M2PS2
//         );
//     }

//     #[test]
//     fn test_einsteins_law() {
//         let mass = 0.1 * KG;

//         let c = 3.0e8 * MPS; // TODO: Replace with constant.

//         assert_eq!(mass * c * c, 9.0e15 * J);
//     }

//     // # TODO: More at:
//     // https://www.jagranjosh.com/general-knowledge/important-laws-of-physics-1513943551-1
// }
