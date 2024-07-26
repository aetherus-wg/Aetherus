//
// Attribute first-stage imager linker.

use crate::{
    err::Error,
    fmt_report,
    math::{Point3, Vec3},
    ord::{Link, Name, Set, cartesian::{X, Y}},
    phys::{Reflectance, SpectrumBuilder},
    sim::attribute::AttributeLinkerLinkerLinkerLinkerLinker,
    tools::{Binner, Range},
};
use arctk_attr::file;
use std::fmt::{Display, Formatter};

/// Surface attribute setup.
/// Handles detector linking.
#[file]
pub enum AttributeLinkerLinkerLinkerLinkerLinkerLinker {
    /// Material interface, inside material name, outside material name.
    Interface(Name, Name),
    /// Partially reflective mirror, reflection fraction.
    Mirror(f64),
    /// Spectrometer id, range, resolution.
    Spectrometer(Name, [f64; 2], usize),
    /// Imager id, resolution, horizontal width (m), center, forward direction.
    Imager(Name, [usize; 2], f64, Point3, Vec3),
    /// Imager id, resolution, horizontal width (m), center, forward direction, wavelength binner (m).
    Ccd(Name, [usize; 2], f64, Point3, Vec3, Binner),
    /// A purely reflecting material, with a provided reflectance model.
    /// The first coefficient is diffuse albedo, the second is specular.
    Reflector(
        Option<SpectrumBuilder>,
        Option<SpectrumBuilder>,
        Option<f64>,
    ),
    /// A photon collector, which collects the photon that interact with the linked entities.
    /// These photons can be optionally killed, or left to keep propogating.
    PhotonCollector(Name, bool),
    AttributeChain(Vec<AttributeLinkerLinkerLinkerLinkerLinker>),
}

impl<'a> Link<'a, usize> for AttributeLinkerLinkerLinkerLinkerLinkerLinker {
    type Inst = AttributeLinkerLinkerLinkerLinkerLinker;

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
            Self::Imager(id, resolution, width, center, forward) => {
                Self::Inst::Imager(id, resolution, width, center, forward)
            }
            Self::Ccd(id, _resolution, width, center, forward, binner) => {
                Self::Inst::Ccd(id, _resolution, width, center, forward, binner)
            }
            Self::Reflector(diff_ref, spec_ref, specularity) => {
                Self::Inst::Reflector(diff_ref, spec_ref, specularity)
            }
            Self::PhotonCollector(ref id, _kill_photons) => {
                Self::Inst::PhotonCollector(id.clone(), _kill_photons)
            },
            Self::AttributeChain(ref attrs) => {
                todo!()
            }
        })
    }
}

impl Display for AttributeLinkerLinkerLinkerLinkerLinkerLinker {
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
            Self::Ccd(ref id, res, width, center, forward, ref binner) => {
                writeln!(fmt, "Ccd: ...")?;
                fmt_report!(fmt, id, "name");
                fmt_report!(fmt, &format!("[{} x {}]", res[X], res[Y]), "resolution");
                fmt_report!(fmt, width, "width (m)");
                fmt_report!(fmt, center, "center (m)");
                fmt_report!(fmt, forward, "forward");
                fmt_report!(fmt, binner, "binner");
                Ok(())
            }
            Self::Reflector(ref diff_ref, ref spec_ref, ref specularity) => {
                writeln!(fmt, "Reflector: ...")?;
                fmt_report!(
                    fmt,
                    if diff_ref.is_some() {
                        format!("{}", diff_ref.as_ref().unwrap())
                    } else {
                        String::from("none")
                    },
                    "diffuse reflectance"
                );
                fmt_report!(
                    fmt,
                    if spec_ref.is_some() {
                        format!("{}", spec_ref.as_ref().unwrap())
                    } else {
                        String::from("none")
                    },
                    "specular reflectance"
                );
                fmt_report!(
                    fmt,
                    if specularity.is_some() {
                        format!("{}", specularity.as_ref().unwrap())
                    } else {
                        String::from("none")
                    },
                    "specularity"
                );
                Ok(())
            }
            Self::PhotonCollector(ref id, ref kill_phot) => {
                writeln!(fmt, "Photon Collector: ...")?;
                fmt_report!(fmt, id, "name");
                fmt_report!(fmt, kill_phot, "kill photons?");
                Ok(())
            },
            Self::AttributeChain(ref attrs) => {
                writeln!(fmt, "Attribute Chain: ...")?;
                for attr in attrs {
                    attr.fmt(fmt)?;
                }
                Ok(())
            }
        }
    }
}
