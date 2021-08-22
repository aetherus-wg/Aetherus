//! International System of Units

use crate::core::dim::dimension::*;

/// Create the constant used to form the SI equivalent base unit.
/// As UCUM uses grams as a base, not kilograms, then some values are scaled to account.
macro_rules! unit {
    ($dim:ident, $unit:ident, $value:expr) => {
        #[allow(non_upper_case_globals)]
        pub const $unit: $dim = $dim {
            value_unsafe: $value,
            _marker: std::marker::PhantomData,
        };
    };
}

unit!(Length, Meter, 1.0);
unit!(Time, Second, 1.0);
unit!(Time, Minutes, 60.0);
unit!(Time, Hour, 3600.0);
unit!(Mass, Kilogram, 1.0e3);
unit!(Charge, Coulomb, 1.0);
unit!(Temperature, Kelvin, 1.0);
unit!(Luminosity, Candela, 1.0);
unit!(Angle, Radian, 1.0);
unit!(Force, Newton, 1.0e3);
unit!(Pressure, Pascal, 1.0e3);
unit!(Energy, Joule, 1.0e3);
unit!(Power, Watt, 1.0e3);
unit!(Current, Ampere, 1.0);
unit!(Voltage, Volt, 1.0e3);
unit!(Capacitance, Farad, 1.0e-3);
unit!(Resistance, Ohm, 1.0e3);
unit!(Conductance, Siemens, 1.0e-3);
unit!(MagFlux, Weber, 1.0e3);
unit!(MagFluxDens, Tesla, 1.0e3);
unit!(Inductance, Henry, 1.0e3);
unit!(AngularFrequency, RadianPerSecond, 1.0);
unit!(Area, Meter2, 1.0);
unit!(Volume, Meter3, 1.0);
unit!(InvLength, PerMeter, 1.0);
unit!(Frequency, Hertz, 1.0);
unit!(Steradian, Steradian, 1.0);
unit!(Velocity, MeterPerSecond, 1.0);
unit!(Acceleration, MeterPerSecond2, 1.0);
unit!(Jerk, MeterPerSecond3, 1.0);
unit!(MassDensity, KilogramPerMeter3, 1.0e3);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_newtons_second_law() {
        let mass = 6.0 * Kilogram;
        let accel = 2.0 * MeterPerSecond2;

        let force = mass * accel;

        assert_eq!(force, 12.0 * Newton);
    }

    #[test]
    fn test_archimedes_principle() {
        let fluid_density = 0.9 * KilogramPerMeter3;
        let grav_accel = 9.81 * MeterPerSecond2;
        let displacement_vol = 1.0 * Meter3;

        let buoyant_force = -fluid_density * grav_accel * displacement_vol;

        assert_eq!(buoyant_force, -8.829 * Newton);
    }

    #[test]
    fn test_ohms_law() {
        let voltage = 20.0 * Volt;
        let resis = 5.0 * Ohm;

        let current = voltage / resis;

        assert_eq!(current, 4.0 * Ampere);
    }

    #[test]
    fn test_coulombs_law() {
        let charge_a = 0.1 * Coulomb;
        let charge_b = 2.5 * Coulomb;
        let distance = 10.0 * Meter;

        let k = 8.9875e9 * Newton * Meter2 / (Coulomb * Coulomb); // TODO: Replace with constant.

        let force = k * (charge_a * charge_b) / (distance * distance);

        assert_eq!(force, 22_468_750.0 * Newton);
    }

    #[test]
    fn test_stefans_law() {
        let area = 2.0 * Meter2;
        let temp = 2000.0 * Kelvin;

        let sigma = 5.6703e-8 * Watt / (Meter2 * Kelvin * Kelvin * Kelvin * Kelvin); // TODO: Replace with constant.

        let lumin = area * sigma * temp * temp * temp * temp;

        assert_eq!(lumin, 1_814_496.0 * Watt);
    }

    #[test]
    fn test_pascals_law() {
        let force = 2000.0 * Newton;
        let area = 2.0 * Meter2;

        let pressure = force / area;

        assert_eq!(pressure, 1000.0 * Pascal);
    }

    #[test]
    fn test_hookes_law() {
        let length = 0.1 * Meter;
        let k = 2.0 * Newton / Meter;

        assert_eq!(-k * length, -0.2 * Newton);
    }

    #[test]
    fn test_bernoullis_law() {
        let vel = 2.0 * MeterPerSecond;
        let pressure = 3.0 * Pascal;
        let density = 0.1 * KilogramPerMeter3;
        let elevation = 1000.0 * Meter;

        let g = 9.81 * MeterPerSecond2; // TODO: Replace with constant.

        assert_eq!(
            ((vel * vel) / 2.0) + (pressure / density) + (g * elevation),
            9842.0 * MeterPerSecond2 * Meter
        );
    }

    #[test]
    fn test_einsteins_law() {
        let mass = 0.1 * Kilogram;

        let c = 3.0e8 * MeterPerSecond; // TODO: Replace with constant.

        assert_eq!(mass * c * c, 9.0e15 * Joule);
    }

    // # TODO: More at:
    // https://www.jagranjosh.com/general-knowledge/important-laws-of-physics-1513943551-1
}
