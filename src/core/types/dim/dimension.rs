//! Physical units

use crate::core::Real;
use dimensioned::{tarr, typenum, ucum, ucum::UCUM};

/// Create the alias for the Unified Code for Units of Measurements (UCUM) dimensions.
/// Note that UCUM does not consider Moles a unit, since it is dimensionless and can be defined in terms of Avogadro's number.
/// Also note that angles are a distinct base dimension, this allows separation of planar angles from steradians.
/// Finally note that electrical charge, rather than current, forms the base unit for electrical phenomena.
/// The underlying type is wrapped the given dimension.
/// Values are stored and manipulated in terms of the given base unit.
macro_rules! dimension {
    ($type:ty, $dim:ident, $unit:ident) => {
        pub type $dim = ucum::$unit<$type>;
    };
    ($type:ty, $dim:ident, [$d0:ident, $d1:ident, $d2:ident, $d3:ident, $d4:ident, $d5:ident, $d6:ident]) => {
        pub type $dim = UCUM<
            $type,
            tarr![
                typenum::$d0,
                typenum::$d1,
                typenum::$d2,
                typenum::$d3,
                typenum::$d4,
                typenum::$d5,
                typenum::$d6
            ],
        >;
    };
}

// Base units
dimension!(Real, Length, Meter);
dimension!(Real, Time, Second);
dimension!(Real, Mass, Gram);
dimension!(Real, Charge, Coulomb);
dimension!(Real, Temperature, Kelvin);
dimension!(Real, Luminosity, Candela);
dimension!(Real, Angle, Radian);

// Derived units.
dimension!(Real, Force, MilliNewton);
dimension!(Real, Pressure, MilliPascal);
dimension!(Real, Energy, MilliJoule);
dimension!(Real, Power, MilliWatt);
dimension!(Real, Current, Ampere);
dimension!(Real, Voltage, MilliVolt);
dimension!(Real, Capacitance, KiloFarad);
dimension!(Real, Resistance, MilliOhm);
dimension!(Real, Conductance, KiloSiemens);
dimension!(Real, MagFlux, MilliWeber);
dimension!(Real, MagFluxDens, MilliTesla);
dimension!(Real, Inductance, MilliHenry);
dimension!(Real, AngularFrequency, [Z0, N1, Z0, Z0, Z0, Z0, P1]);
dimension!(Real, Area, Meter2);
dimension!(Real, Volume, Meter3);
dimension!(Real, InvLength, PerMeter);
dimension!(Real, Frequency, Hertz);
dimension!(Real, Steradian, Steradian);
dimension!(Real, Velocity, MeterPerSecond);
dimension!(Real, Acceleration, MeterPerSecond2);
dimension!(Real, Jerk, MeterPerSecond3);
dimension!(Real, MassDensity, [N3, Z0, P1, Z0, Z0, Z0, Z0]);
