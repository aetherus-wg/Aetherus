//! International System of Units
//! This module contains the complementary SI prefixes for the units in core::dim::si;
//! Some examples of their use:
//! ```rust
//! # use aetherus::core::dim::{si::*, prefix::*};
//! let equatorial_radius_of_earth = 6_378. * KILO * Meter;
//! let equatorial_radius_of_earth = 6_378. * KILO * Meter;
//! ```

use crate::core::Real;

pub const TERA: Real = 1.0e12;
pub const GIGA: Real = 1.0e9;
pub const MEGA: Real = 1.0e6;
pub const KILO: Real = 1.0e3;
pub const HECTO: Real = 1.0e2;
pub const DECA: Real = 1.0e1;
pub const DECI: Real = 1.0e-1;
pub const CENTI: Real = 1.0e-2;
pub const MILLI: Real = 1.0e-3;
pub const MICRO: Real = 1.0e-6;
pub const NANO: Real = 1.0e-9;
pub const PICO: Real = 1.0e-12;

#[cfg(test)]
mod tests {
    use crate::core::dim::{prefix::*, si::*};
    use assert_approx_eq::assert_approx_eq;
    use dimensioned::Dimensioned;

    /// A collection of unit comparisons to check that the SI prefixes are working.
    /// These tests are sorted in descending order of prefix magnitude.
    #[test]
    fn test_si_prefixes() {
        // An astronomical unit
        assert_eq!(0.149597870700 * TERA * Meter, 149_597_870_700. * Meter); // Meter

        // A bolt of lightning!
        assert_eq!(1.21 * GIGA * Watt, 1.21E9 * Watt);

        // The neutral hydrogen emission line.
        assert_eq!(1420.406 * MEGA * Hertz, 1.420406E9 * Hertz);

        // Radius of the Earth - Distance.
        assert_eq!(6_378. * KILO * Meter, 6.378E6 * Meter);

        // Mean atmospheric pressure at sea level for Earth.
        assert_eq!(1013.25 * HECTO * Pascal, 101_325. * Pascal);

        // Some misellaneous tests for these intermediate prefixes, as I couldn't thinkg of anything creative.
        // Please feel free to adapt them if you can think of anything better.
        assert_eq!(1. * DECA * Meter, 10. * Meter);
        assert_eq!(1. * DECI * Meter, 0.1 * Meter);
        assert_eq!(1. * CENTI * Meter, (1.0 / 100.0) * Meter);
        assert_eq!(1. * MILLI * Meter, (1.0 / 1000.0) * Meter);

        // Wavelength of the Brackett Gamma transition line.
        let micro_left = 2.1655 * MICRO * Meter;
        let micro_right = 2.1655E-6 * Meter;
        assert_approx_eq!(*micro_left.value_unsafe(), *micro_right.value_unsafe());

        // Blackbody peak of the Sun, assuming Teff = 5772K (IAU).
        assert_eq!(502. * NANO * Meter, 5.02E-7 * Meter);

        // Covalent radius of Hydrogen.
        let pico_left = 31. * PICO * Meter;
        let pico_right = 3.1E-11 * Meter;
        assert_approx_eq!(*pico_left.value_unsafe(), *pico_right.value_unsafe());
    }
}
