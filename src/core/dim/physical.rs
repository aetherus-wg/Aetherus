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
/// Length is a measure of distance.
/// This is a fundamental dimension.
/// Units are the same as SI, meters.
dimension!(Real, Length, Meter);

/// Length is a measure of duration.
/// This is a fundamental dimension.
/// Units are the same as SI, seconds.
dimension!(Real, Time, Second);

/// Mass is a measure of resistance to acceleration within the Higgs field.
/// This is a fundamental dimension.
/// Units are the different to SI, as UCUM base units do not have prefixes.
/// Units are grams.
dimension!(Real, Mass, Gram);

/// Charge is a measure of resistance to acceleration within a electromagnetic field.
/// Units are the different to SI, as charge is considered more fundamental than current (movement of charge).
/// This is a fundamental dimension.
/// Units are coulombs.
dimension!(Real, Charge, Coulomb);

/// Temperature is a measure of the average kinetic energy of an ensemble of particles.
/// This is a convenience dimension.
/// Units are the same as SI, Kelvin.
dimension!(Real, Temperature, Kelvin);

/// Luminosity is a measure of the human eye's perceived response to light intensity.
/// This is a convenience dimension.
/// Units are the same as SI, candela.
dimension!(Real, Luminosity, Candela);

/// Angle is a measure of rotation required to bring one line into coincidence with another.
/// This is a fundamental theoretical dimension.
/// Units are radians.
dimension!(Real, Angle, Radian);

// Power
dimension!(Real, Area, Meter2);
dimension!(Real, Volume, Meter3);

dimension!(Real, Steradian, Steradian);
