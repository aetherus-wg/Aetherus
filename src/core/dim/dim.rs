use crate::core::Real;
use dimensioned::si;
use nalgebra::{Point2, Point3, Point4};




pub const M: Length = si::M;
#[allow(non_upper_case_globals)]
pub const Kg: Mass = si::KG;
#[allow(non_upper_case_globals)]
pub const Sec: Time = si::S;
#[allow(non_upper_case_globals)]
pub const Min: Time = si::MIN;
#[allow(non_upper_case_globals)]
pub const Hrs: Time = si::HR;
pub const A: Current = si::A;
pub const K: Temp = si::K;
#[allow(non_upper_case_globals)]
pub const Cd: Candela = si::CD;
#[allow(non_upper_case_globals)]
pub const Mol: Mole = si::MOL;

pub type Frequency = si::Hertz<Real>;
pub type Force = si::Newton<Real>;
pub type Pressure = si::Pascal<Real>;
pub type Energy = si::Joule<Real>;
pub type Power = si::Watt<Real>;
pub type Charge = si::Coulomb<Real>;


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
