//! Attribute first-stage imager linker.

use crate::{
    err::Error,
    fmt_report,
    math::{Point3, Vec3},
    ord::{Link, Name, Set, X, Y},
    phys::{Reflectance, SpectrumBuilder},
    sim::{attribute::AttributeLinkerLinkerLinkerLinker},
    tools::{Binner, Range},
};
use arctk_attr::file;
use std::fmt::{Display, Formatter};

/// Surface attribute setup.
/// Handles detector linking.
#[file]
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
    Reflector(Option<SpectrumBuilder>, Option<SpectrumBuilder>, Option<f64>),
    /// A photon collector, which collects the photon that interact with the linked entities.
    /// These photons can be optionally killed, or left to keep propogating. 
    PhotonCollector(Name, bool)
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
            Self::Ccd(id, _resolution, width, center, forward, binner) => Self::Inst::Ccd(id, _resolution, width, center, forward, binner),
            Self::Reflector(diff_ref, spec_ref, diff_spec_ratio) => {
                let ref_model = if diff_ref.is_some() {
                    if spec_ref.is_some() {
                        // Check that the diffuse-specular ratio has been defined. 
                        assert!(diff_spec_ratio.is_some());
                        Reflectance::Composite {
                            diffuse_refspec: diff_ref.unwrap().build()?,
                            specular_refspec: spec_ref.unwrap().build()?,
                            diffuse_specular_ratio: diff_spec_ratio.unwrap(),
                        }
                    } else {
                        Reflectance::Lambertian { refspec: diff_ref.unwrap().build()? }
                    }
                } else {
                    Reflectance::Specular { refspec: spec_ref.unwrap().build()? }
                };

                Self::Inst::Reflector(ref_model)
            },
            Self::PhotonCollector(ref id, _kill_photons) => {
                Self::Inst::PhotonCollector(*reg.get(&id).unwrap_or_else(|| panic!("Failed to link attribute-photon collector key : {}", id)))
            }
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
            Self::Reflector(ref diff_ref, ref spec_ref, ref diff_spec_ratio) => {
                writeln!(fmt, "Reflector: ...")?;
                fmt_report!(fmt, if diff_ref.is_some() { format!("{}", diff_ref.as_ref().unwrap()) } else { String::from("none") }, "diffuse reflectance");
                fmt_report!(fmt, if spec_ref.is_some() { format!("{}", spec_ref.as_ref().unwrap()) } else { String::from("none") }, "specular reflectance");
                fmt_report!(fmt, if diff_spec_ratio.is_some() { format!("{}", diff_spec_ratio.as_ref().unwrap()) } else { String::from("none") }, "diffuse-to-specular ratio");
                Ok(())
            },
            Self::PhotonCollector(ref id, ref kill_phot) => {
                writeln!(fmt, "Photon Collector: ...")?;
                fmt_report!(fmt, id, "name");
                fmt_report!(fmt, kill_phot, "kill photons?");
                Ok(())
            },
        }
    }
}
