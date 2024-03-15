//! Attribute third-stage material linker.

use crate::{
    err::Error,
    fmt_report,
    geom::Orient,
    core::{Link, Name, Set},
    phys::Material,
    phys::Reflectance,
    sim::attribute::Attribute,
    core::Binner,
};
use std::fmt::{Display, Formatter};

/// Surface attribute setup.
pub enum AttributeLinker {
    /// Material interface, inside material name, outside material name.
    Interface(Name, Name),
    /// Partially reflective mirror, reflection fraction.
    Mirror(f64),
    /// Spectrometer id.
    Spectrometer(usize),
    /// Imager id, width, orientation.
    Imager(usize, f64, Orient),
    /// CCD detector id, width, orientation, binner.
    Ccd(usize, f64, Orient, Binner),
    /// A purely reflecting material, with a provided reflectance model.
    Reflector(Reflectance),
    /// A photon collector, which collects the photon that interact with the linked entities.
    /// These photons can be optionally killed, or left to keep propogating.
    PhotonCollector(usize),
}

impl<'a> Link<'a, Material> for AttributeLinker {
    type Inst = Attribute<'a>;

    #[inline]
    fn requires(&self) -> Vec<Name> {
        match *self {
            Self::Interface(ref inside, ref outside) => vec![inside.clone(), outside.clone()],
            Self::Mirror(..)
            | Self::Spectrometer(..)
            | Self::Imager(..)
            | Self::Ccd(..)
            | Self::Reflector(..)
            | Self::PhotonCollector(..) => {
                vec![]
            }
        }
    }

    #[inline]
    fn link(self, mats: &'a Set<Material>) -> Result<Self::Inst, Error> {
        Ok(match self {
            Self::Interface(ref inside, ref outside) => Self::Inst::Interface(
                mats.get(inside).unwrap_or_else(|| {
                    panic!("Failed to link attribute-interface key: {}", inside)
                }),
                mats.get(outside).unwrap_or_else(|| {
                    panic!("Failed to link attribute-interface key: {}", outside)
                }),
            ),
            Self::Mirror(r) => Self::Inst::Mirror(r),
            Self::Spectrometer(id) => Self::Inst::Spectrometer(id),
            Self::Imager(id, width, orient) => Self::Inst::Imager(id, width, orient),
            Self::Ccd(id, width, orient, binner) => Self::Inst::Ccd(id, width, orient, binner),
            Self::Reflector(reflectance) => Self::Inst::Reflector(reflectance),
            Self::PhotonCollector(id) => Self::Inst::PhotonCollector(id),
        })
    }
}

impl Display for AttributeLinker {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            Self::Interface(ref in_mat, ref out_mat) => {
                write!(fmt, "Interface: {} :| {}", in_mat, out_mat)
            }
            Self::Mirror(abs) => {
                write!(fmt, "Mirror: {}% abs", abs * 100.0)
            }
            Self::Spectrometer(id) => {
                write!(fmt, "Spectrometer: {}", id)
            }
            Self::Imager(ref id, width, ref orient) => {
                writeln!(fmt, "Imager: ...")?;
                fmt_report!(fmt, id, "name");
                fmt_report!(fmt, width, "width (m)");
                fmt_report!(fmt, orient, "orientation");
                Ok(())
            }
            Self::Ccd(ref id, width, ref orient, ref binner) => {
                writeln!(fmt, "Ccd: ...")?;
                fmt_report!(fmt, id, "name");
                fmt_report!(fmt, width, "width (m)");
                fmt_report!(fmt, orient, "orientation");
                fmt_report!(fmt, binner, "binner");
                Ok(())
            }
            Self::Reflector(ref reflectance) => {
                writeln!(fmt, "Reflector: ...")?;
                fmt_report!(fmt, reflectance, "reflectance");
                Ok(())
            }
            Self::PhotonCollector(ref id) => {
                writeln!(fmt, "Photon Collector: ...")?;
                fmt_report!(fmt, id, "name");
                Ok(())
            }
        }
    }
}
