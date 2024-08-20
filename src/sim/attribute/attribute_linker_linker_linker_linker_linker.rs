//! Attribute first-stage imager linker.

use crate::{
    err::Error,
    fmt_report,
    math::{Point3, Vec3},
    ord::{cartesian::{X, Y}, Link, Name, Set},
    phys::{ReflectanceBuilder, ReflectanceBuilderShim},
    sim::attribute::AttributeLinkerLinkerLinkerLinker,
    tools::{Binner, Range},
    io::output::Rasteriser,
};
use arctk_attr::file;
use std::fmt::{Display, Formatter};

/// Surface attribute setup.
/// Handles detector linking.
#[derive(Clone)]
pub enum AttributeLinkerLinkerLinkerLinkerLinker {
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
    Reflector(ReflectanceBuilderShim),
    /// A photon collector, which collects the photon that interact with the linked entities.
    /// These photons can be optionally killed, or left to keep propogating.
    PhotonCollector(Name, bool),
    /// A chain of attributes where are executed in order. 
    AttributeChain(Vec<AttributeLinkerLinkerLinkerLinkerLinker>),
    /// An output into the output plane object. This rasterises the photon packet into plane. 
    Rasterise(usize, Rasteriser),
}

impl<'a> Link<'a, usize> for AttributeLinkerLinkerLinkerLinkerLinker {
    type Inst = AttributeLinkerLinkerLinkerLinker;

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
            Self::Reflector(ref_sim) => {
                let ref_build: ReflectanceBuilder = ref_sim.into();
                let ref_model = ref_build.build()?;
                Self::Inst::Reflector(ref_model)
            }
            Self::PhotonCollector(ref id, _kill_photons) => {
                Self::Inst::PhotonCollector(*reg.get(&id).unwrap_or_else(|| {
                    panic!("Failed to link attribute-photon collector key : {}", id)
                }))
            },
            Self::AttributeChain(attrs) => {
                let linked_attrs: Result<Vec<_>, _> = attrs.iter()
                    .map(|a| a.clone().link(&reg))
                    .collect();

                Self::Inst::AttributeChain(linked_attrs?)
            }
            Self::Rasterise(id, rast) => Self::Inst::Rasterise(id, rast),
        })
    }
}

impl Display for AttributeLinkerLinkerLinkerLinkerLinker {
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
            Self::Reflector(ref ref_shim) => {
                writeln!(fmt, "Reflector: ...")?;
                fmt_report!(
                    fmt,
                    if ref_shim.0.is_some() {
                        format!("{}", ref_shim.0.as_ref().unwrap())
                    } else {
                        String::from("none")
                    },
                    "diffuse reflectance"
                );
                fmt_report!(
                    fmt,
                    if ref_shim.1.is_some() {
                        format!("{}", ref_shim.1.as_ref().unwrap())
                    } else {
                        String::from("none")
                    },
                    "specular reflectance"
                );
                fmt_report!(
                    fmt,
                    if ref_shim.2.is_some() {
                        format!("{}", ref_shim.2.as_ref().unwrap())
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