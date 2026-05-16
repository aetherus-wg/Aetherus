//! Optical attributes.

use crate::{
    fmt_report,
    phys::{Material, Reflectance},
};
use std::fmt::{Display, Error, Formatter};

/// Surface attributes.
#[derive(Debug, PartialEq, Clone)]
pub enum Attribute {
    /// Material interface, inside material reference, outside material reference.
    Interface(Material, Material),
    /// A purely reflecting material, with a provided reflectance model.
    Reflector(Reflectance),
    /// Partially reflective mirror, reflection fraction.
    Mirror(f64),
    /// A photon collector, which collects the photon that interact with the linked entities.
    /// These photons can be optionally killed, or left to keep propogating.
    Detector(usize),
    /// A chain of attributes, allowing us to perform multiple actions with a
    /// photon packet for each interaction. We can chain attributes together here.
    AttributeChain(Vec<Attribute>),
}

impl Display for Attribute {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Self::Interface(in_mat, out_mat) => {
                write!(fmt, "Interface: {} :| {}", in_mat, out_mat)
            }
            Self::Mirror(abs) => {
                write!(fmt, "Mirror: {}% abs", abs * 100.0)
            }
            Self::Reflector(ref reflectance) => {
                writeln!(fmt, "Reflector: ...")?;
                fmt_report!(fmt, reflectance, "reflectance");
                Ok(())
            }
            Self::Detector(ref id) => {
                writeln!(fmt, "Photon Collector: ...")?;
                fmt_report!(fmt, id, "name");
                Ok(())
            },
            Self::AttributeChain(ref attrs) => {
                writeln!(fmt, "Attribute Chain: ...")?;
                for attr in attrs {
                    attr.fmt(fmt)?;
                }
                Ok(())
            },
        }
    }
}
