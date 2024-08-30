//! Attribute first-stage imager linker.

use crate::{
    err::Error,
    fmt_report,
    geom::{Orient, Ray},
    math::{Dir3, Point3, Vec3},
    ord::{Link, Name, Set, cartesian::{X, Y}},
    phys::Reflectance,
    sim::attribute::AttributeLinkerLinker,
    tools::{Binner, Range},
    io::output::{Rasteriser, AxisAlignedPlane},
};
use std::fmt::{Display, Formatter};

/// Surface attribute setup.
/// Handles detector linking.
#[derive(Clone)]
pub enum AttributeLinkerLinkerLinker {
    /// Material interface, inside material name, outside material name.
    Interface(Name, Name),
    /// Partially reflective mirror, reflection fraction.
    Mirror(f64),
    /// Spectrometer id, range, resolution.
    Spectrometer(Name, [f64; 2], usize),
    /// Imager id, resolution, horizontal width (m), center, forward direction.
    Imager(Name, [usize; 2], f64, Point3, Vec3),
    /// CCD detector id, width, orientation, binner.
    Ccd(usize, f64, Orient, Binner),
    /// A purely reflecting material, with a provided reflectance model.
    Reflector(Reflectance),
    /// A photon collector, which collects the photon that interact with the linked entities.
    /// These photons can be optionally killed, or left to keep propogating.
    PhotonCollector(usize),
    /// A chain of attributes where are executed in order. 
    AttributeChain(Vec<AttributeLinkerLinkerLinker>),
    /// An output into the output plane object. This rasterises the photon packet into plane. 
    Rasterise(usize, Rasteriser),
    /// Hyperspectral output - output into a volume output
    Hyperspectral(usize, AxisAlignedPlane),
    
}

impl<'a> Link<'a, usize> for AttributeLinkerLinkerLinker {
    type Inst = AttributeLinkerLinker;

    #[inline]
    fn requires(&self) -> Vec<Name> {
        vec![]
    }

    #[inline]
    fn link(self, reg: &'a Set<usize>) -> Result<Self::Inst, Error> {
        Ok(match self {
            Self::Interface(inside, outside) => Self::Inst::Interface(inside, outside),
            Self::Mirror(r) => Self::Inst::Mirror(r),
            Self::Spectrometer(name, range, resolution) => {
                Self::Inst::Spectrometer(name, range, resolution)
            }
            Self::Imager(id, _resolution, width, center, forward) => Self::Inst::Imager(
                *reg.get(&id)
                    .unwrap_or_else(|| panic!("Failed to link attribute-imager key: {}", id)),
                width,
                Orient::new(Ray::new(center, Dir3::from(forward))),
            ),
            Self::Ccd(id, width, orient, binner) => Self::Inst::Ccd(id, width, orient, binner),
            Self::Reflector(reflectance) => Self::Inst::Reflector(reflectance),
            Self::PhotonCollector(id) => Self::Inst::PhotonCollector(id),
            Self::AttributeChain(attrs) => {
                let linked_attrs: Result<Vec<_>, _> = attrs.iter()
                    .map(|a| a.clone().link(&reg))
                    .collect();

                Self::Inst::AttributeChain(linked_attrs?)
            }
            Self::Rasterise(id, rast) => Self::Inst::Rasterise(id, rast),
            Self::Hyperspectral(id, plane) => Self::Inst::Hyperspectral(id, plane),
        })
    }
}

impl Display for AttributeLinkerLinkerLinker {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            Self::Interface(ref in_mat, ref out_mat) => {
                write!(fmt, "Interface: {} :| {}", in_mat, out_mat)
            }
            Self::Mirror(abs) => {
                write!(fmt, "Mirror: {}% abs", abs * 100.0)
            }
            Self::Spectrometer(ref id, [min, max], bins) => {
                write!(
                    fmt,
                    "Spectrometer: {} {} ({})",
                    id,
                    Range::new(min, max),
                    bins
                )
            }
            Self::Imager(ref id, res, width, center, forward) => {
                writeln!(fmt, "Imager: ...")?;
                fmt_report!(fmt, id, "name");
                fmt_report!(fmt, &format!("[{} x {}]", res[X], res[Y]), "resolution");
                fmt_report!(fmt, width, "width (m)");
                fmt_report!(fmt, center, "center (m)");
                fmt_report!(fmt, forward, "forward");
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
            Self::Hyperspectral(ref id, ref plane) => {
                writeln!(fmt, "Hyperspectral: ...")?;
                fmt_report!(fmt, id, "name");
                fmt_report!(fmt, plane, "plane");
                Ok(())
            }
        }
    }
}
