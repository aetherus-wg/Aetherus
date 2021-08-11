//! International System of Units

use crate::core::Real;

macro_rules! dimension {
    ($underlying:ty, $dim:ident, $unit:ident, $suffix:ident) => {
        pub type $dim = dimensioned::si::$unit<$underlying>;

        #[allow(dead_code)]
        const $suffix: $dim = dimensioned::si::$suffix;
    };
}

dimension!(Real, Length, Meter, M);
dimension!(Real, Mass, Kilogram, KG);
// dimension!(Real, Time, Second, S);
dimension!(Real, Current, Ampere, A);
dimension!(Real, Temp, Kelvin, K);
// dimension!(Real, Lumin, Candela);
// dimension!(Real, Mole, Mole);
// dimension!(Real, Freq, Hertz);
dimension!(Real, Force, Newton, N);
dimension!(Real, Pressure, Pascal, PA);
dimension!(Real, Energy, Joule, J);
dimension!(Real, Power, Watt, W);
dimension!(Real, Charge, Coulomb, C);
dimension!(Real, ElecPotential, Volt, V);
// dimension!(Real, Capacitance, Farad);
dimension!(Real, Resistance, Ohm, OHM);
// dimension!(Real, Conductance, Siemens);
// dimension!(Real, MagFlux, Weber);
// dimension!(Real, MagFluxDensity, Tesla);
// dimension!(Real, Inductance, Henry);
// dimension!(Real, Candela, Lumen);
// dimension!(Real, Illuminance, Lux);
dimension!(Real, Area, Meter2, M2);
dimension!(Real, Volume, Meter3, M3);
dimension!(Real, Velocity, MeterPerSecond, MPS);
dimension!(Real, Acceleration, MeterPerSecond2, MPS2);
// dimension!(Real, Jerk, MeterPerSecond3);
dimension!(Real, MassDensity, KilogramPerMeter3, KGPM3);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_newtons_second_law() {
        let mass = 6.0 * KG;
        let accel = 2.0 * MPS2;

        let force = mass * accel;

        assert_eq!(force, 12.0 * N);
    }

    #[test]
    fn test_archimedes_principle() {
        let fluid_density = 0.9 * KGPM3;
        let grav_accel = 9.81 * MPS2;
        let displacement_vol = 1.0 * M3;

        let buoyant_force = -fluid_density * grav_accel * displacement_vol;

        assert_eq!(buoyant_force, -8.829 * N);
    }

    #[test]
    fn test_ohms_law() {
        let voltage = 20.0 * V;
        let resis = 5.0 * OHM;

        let current = voltage / resis;

        assert_eq!(current, 4.0 * A);
    }

    #[test]
    fn test_coulombs_law() {
        let charge_a = 0.1 * C;
        let charge_b = 2.5 * C;
        let distance = 10.0 * M;

        let k = 8.9875e9 * N * M * M / (C * C); // TODO: Replace with constant.

        let force = k * (charge_a * charge_b) / (distance * distance);

        assert_eq!(force, 22468750.0 * N);
    }

    #[test]
    fn test_stefans_law() {
        let area = 2.0 * M2;
        let temp = 2000.0 * K;

        let sigma = 5.6703e-8 * W / (M * M * K * K * K * K); // TODO: Replace with constant.

        let lumin = area * sigma * temp * temp * temp * temp;

        assert_eq!(lumin, 1814496.0 * W);
    }

    #[test]
    fn test_pascals_law() {
        let force = 2000.0 * N;
        let area = 2.0 * M2;

        let pressure = force / area;

        assert_eq!(pressure, 1000.0 * PA);
    }

    #[test]
    fn test_einsteins_law() {
        let mass = 0.1 * KG;

        let c = 3.0e8 * MPS; // TODO: Replace with constant.

        assert_eq!(mass * c * c, 9.0e15 * J);
    }

    // # TODO: More at:
    // https://www.jagranjosh.com/general-knowledge/important-laws-of-physics-1513943551-1
}
