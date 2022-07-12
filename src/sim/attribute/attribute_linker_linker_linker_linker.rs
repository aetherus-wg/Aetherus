//! Attribute first-stage imager linker.

use crate::{
    err::Error,
    fmt_report,
    geom::{Orient, Ray},
    math::{Dir3, Point3, Vec3},
    ord::{Link, Name, Set, X, Y},
    sim::{attribute::AttributeLinkerLinkerLinker},
    phys::Reflectance,
    tools::{Binner, Range},
};
use arctk_attr::file;
use std::fmt::{Display, Formatter};

/// Surface attribute setup.
/// Handles detector linking.
#[file]
pub enum AttributeLinkerLinkerLinkerLinker {
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
    Reflector(f64, f64, f64),
}

impl<'a> Link<'a, usize> for AttributeLinkerLinkerLinkerLinker {
    type Inst = AttributeLinkerLinkerLinker;

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
            Self::Ccd(id, _resolution, width, center, forward, binner) => Self::Inst::Ccd(
                *reg.get(&id)
                    .unwrap_or_else(|| panic!("Failed to link attribute-ccd key: {}", id)),
                width,
                Orient::new(Ray::new(center, Dir3::from(forward))),
                binner,
            ),
            Self::Reflector(diff_alb, spec_alb, spec_diff_ratio) => {
                let ref_model = if diff_alb > 0.0 {
                    if spec_alb > 0.0 {
                        Reflectance::Composite { diffuse_albedo: diff_alb, specular_albedo: spec_alb, specular_diffuse_ratio: spec_diff_ratio }
                    } else {
                        Reflectance::Lambertian { albedo: diff_alb }
                    }
                } else {
                    Reflectance::Specular { albedo: spec_alb }
                }; 

                Self::Inst::Reflector(ref_model)
            },
        })
    }
}

impl Display for AttributeLinkerLinkerLinkerLinker {
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
            Self::Reflector(ref diff_alb, ref spec_alb, ref spec_diff_ratio) => {
                writeln!(fmt, "Reflector: ...")?;
                fmt_report!(fmt, diff_alb, "diffuse albedo");
                fmt_report!(fmt, spec_alb, "specular albedo");
                fmt_report!(fmt, spec_diff_ratio, "specular-to-diffuse ratio");
                Ok(())
            }
        }
    }
}
