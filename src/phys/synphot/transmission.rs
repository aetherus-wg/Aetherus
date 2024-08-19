use crate::{
    fmt_report, math::Probability, phys::Photon
};
use std::fmt::{Display, Formatter};


#[derive(Debug, Clone, PartialEq)]
pub struct Transmission {
    pub spec: Probability,
}

impl Transmission {
    pub fn sample(&self, phot: &Photon) -> f64 {
        self.spec.sample_at(phot.wavelength())
    }
}

impl Display for Transmission {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(fmt, "Transmission: ")?;
        fmt_report!(fmt, self.spec, "response");
        Ok(())
    }
}