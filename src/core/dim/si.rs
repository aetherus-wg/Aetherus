//! International System of Units

use crate::core::Real;
use dimensioned::si;

macro_rules! dimension {
    ($underlying:ty, $dim:ident, $unit:ident) => {
        pub type $dim = si::$unit<$underlying>;
    };

}

dimension!(Real, Length, Meter);
dimension!(Real, Mass, Kilogram);
dimension!(Real, Time, Second);
dimension!(Real, Curr, Ampere);
dimension!(Real, Temp, Kelvin);
dimension!(Real, Lumin, Candela);
dimension!(Real, Mole, Mole);
dimension!(Real, Freq, Hertz);
dimension!(Real, Force, Newton);
dimension!(Real, Pressure, Pascal);
dimension!(Real, Energy, Joule);
dimension!(Real, Power, Watt);
dimension!(Real, Charge, Coulomb);
dimension!(Real, ElecPotential, Volt);
dimension!(Real, Capacitance, Farad);
dimension!(Real, Resistance, Ohm);
dimension!(Real, Conductance, Siemens);
dimension!(Real, MagFlux, Weber);
dimension!(Real, MagFluxDensity, Tesla);
dimension!(Real, Inductance, Henry);
dimension!(Real, Candela, Lumen);
dimension!(Real, Illuminance, Lux);
dimension!(Real, Area, Meter2);
dimension!(Real, Volume, Meter3);
dimension!(Real, Velocity, MeterPerSecond);
dimension!(Real, Acceleration, MeterPerSecond2);
dimension!(Real, Jerk, MeterPerSecond3);

// dimension!(Point3<Real>, Pos3, Meter);


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        // let x = Length::new(6.0);
        let x = 6.0 * Meters;
        let y = x / 1.0 * (M/S);

        assert_eq!(x, Length::new(6.0));
    }
}
