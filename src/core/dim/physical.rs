//! Physical units

use crate::core::Real;

/// Create the alias for the Unified Code for Units of Measurements (UCUM) dimensions.
/// Note that UCUM does not consider Moles a unit, since it is dimensionless and can be defined in terms of Avogadro's number.
/// Also note that angles are a distinct base dimension, this allows separation of planar angles from steradians.
/// Finally note that electrical charge, rather than current, forms the base unit for electrical phenomena.
/// The underlying type is wrapped the given dimension.
/// Values are stored and manipulated in terms of the given base unit.
macro_rules! dimension {
    ($type:ty, $dim:ident, $unit:ident) => {
        pub type $dim = dimensioned::ucum::$unit<$type>;
    };
}

// Base units
dimension!(Real, Length, Meter);
dimension!(Real, Time, Second);
dimension!(Real, Mass, Gram);
dimension!(Real, Charge, Coulomb);
dimension!(Real, Temperature, Kelvin);dimension!(Real, Luminosity, Candela);
dimension!(Real, Angle, Radian);

// Power
dimension!(Real, Area, Meter2);
dimension!(Real, Volume, Meter3);
dimension!(Real, Steradian, Steradian);
