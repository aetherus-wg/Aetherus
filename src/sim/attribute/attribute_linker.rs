//! Attribute third-stage material linker.

use crate::{
    err::Error,
    fmt_report,
    geom::Orient,
    ord::{Link, Name, Set},
    phys::Material,
    phys::Reflectance,
    sim::attribute::Attribute,
    tools::Binner,
    io::output::Rasteriser,
};
use std::fmt::{Display, Formatter};

/// Surface attribute setup.
#[derive(Clone)]
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
    /// A chain of attributes where are executed in order. 
    AttributeChain(Vec<AttributeLinker>),
    /// An output into the output plane object. This rasterises the photon packet into plane. 
    Rasterise(usize, Rasteriser),
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
            | Self::PhotonCollector(..) 
            | Self::AttributeChain(..) 
            | Self::Rasterise(..) => {
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
            Self::AttributeChain(attrs) => {
                let linked_attrs: Result<Vec<_>, _> = attrs.iter()
                    .map(|a| a.clone().link(&mats))
                    .collect();

                Self::Inst::AttributeChain(linked_attrs?)
            }
            Self::Rasterise(id, rast) => Self::Inst::Rasterise(id, rast),
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
            Self::AttributeChain(ref attrs) => {
                writeln!(fmt, "Attribute Chain: ...")?;
                for attr in attrs {
                    attr.fmt(fmt)?;
                }
                Ok(())
            }
            Self::Rasterise(ref id, ref rast) => {
                writeln!(fmt, "Rasterise: ...")?;
                fmt_report!(fmt, id, "name");
                fmt_report!(fmt, rast, "rasteriser");
                Ok(())
            }
        }
    }
}
