//! Optical attributes.

use aetherus_events::mcrt::SrcId;

use crate::{fmt_report, geom::Orient, io::output::{Rasteriser, AxisAlignedPlane}, phys::{Material, Reflectance}, tools::Binner};
use std::fmt::{Display, Error, Formatter};

// TODO: Perhaps pass SurfaceAttr instead of Object as tag to Mesh
pub struct SurfaceAttr {
    src_id: SrcId,
    surf_attr: Attribute
}

/// Surface attributes.
#[derive(Debug, PartialEq, Clone)]
pub enum Attribute {
    /// Material interface, inside material reference, outside material reference.
    Interface(Material, Material),
    /// Partially reflective mirror, reflection fraction.
    Mirror(f64),
    /// Spectrometer detector.
    Spectrometer(usize),
    /// Imager detector id, width, orientation.
    Imager(usize, f64, Orient),
    /// CCD detector id, width, orientation, binner.
    Ccd(usize, f64, Orient, Binner),
    /// A purely reflecting material, with a provided reflectance model.
    Reflector(Reflectance),
    /// A photon collector, which collects the photon that interact with the linked entities.
    /// These photons can be optionally killed, or left to keep propogating.
    PhotonCollector(usize),
    /// A chain of attributes, allowing us to perform multiple actions with a
    /// photon packet for each interaction. We can chain attributes together here.
    AttributeChain(Vec<Attribute>),
    /// An output into the output plane object. This rasterises the photon packet into plane.
    Rasterise(usize, Rasteriser),
    /// Hyperspectral output - output into a volume output
    Hyperspectral(usize, AxisAlignedPlane),
}

impl Display for Attribute {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Self::Interface(in_mat, out_mat) => {
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
                fmt_report!(fmt, binner, "binner (m)");
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
            },
            Self::AttributeChain(ref attrs) => {
                writeln!(fmt, "Attribute Chain: ...")?;
                for attr in attrs {
                    attr.fmt(fmt)?;
                }
                Ok(())
            },
            Self::Rasterise(ref id, ref rast) => {
                writeln!(fmt, "Rasterise: ...")?;
                fmt_report!(fmt, id, "name");
                fmt_report!(fmt, rast, "rasteriser");
                Ok(())
            }
            Self::Hyperspectral(ref id, ref plane) => {
                writeln!(fmt, "Hyperspectral: ...")?;
                fmt_report!(fmt, id, "name");
                fmt_report!(fmt, plane, "plane");
                Ok(())
            }
        }
    }
}
