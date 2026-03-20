//! Material builder.

use crate::{
    err::Error,
    fmt_report,
    math::FormulaBuilder,
    ord::{Build, Name},
    phys::Material,
};
use arctk_attr::file;
use std::fmt::{Display, Formatter};

/// Loadable material.
#[file]
#[derive(Clone)]
pub struct MaterialBuilder {
    /// Refractive index.
    ref_index: FormulaBuilder,
    /// Scattering coefficient [1/m].
    scat_coeff: FormulaBuilder,
    /// Absorption coefficient [1/m].
    abs_coeff: Option<FormulaBuilder>,
    /// Shifting coefficient [1/m].
    shift_coeff: Option<FormulaBuilder>,
    /// Asymmetry factor.
    asym_fact: FormulaBuilder,
}

impl Build for MaterialBuilder {
    type Inst = Material;
    type MetaInfo = Name;

    fn build(self, id: Self::MetaInfo) -> Result<Self::Inst, Error> {
        let ref_index = self.ref_index.build(id.clone())?;
        let scat_coeff = self.scat_coeff.build(id.clone())?;
        let abs_coeff = self
            .abs_coeff
            .map(|abs_coeff| abs_coeff.build(id.clone()))
            .transpose()?;
        let shift_coeff = self
            .shift_coeff
            .map(|shift_coeff| shift_coeff.build(id.clone()))
            .transpose()?;
        let asym_fact = self.asym_fact.build(id.clone())?;

        Ok(Self::Inst::new(
            ref_index,
            scat_coeff,
            abs_coeff,
            shift_coeff,
            asym_fact,
        ))
    }
}

impl Display for MaterialBuilder {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        writeln!(fmt, "...")?;
        fmt_report!(fmt, self.ref_index, "refractive index");
        fmt_report!(fmt, self.scat_coeff, "scattering coefficient (m^-1)");

        let abs_coeff = if let Some(ref abs_coeff) = self.shift_coeff {
            format!("{}", abs_coeff)
        } else {
            "NONE".to_owned()
        };
        fmt_report!(fmt, abs_coeff, "absorption coefficient (m^-1)");

        let shift_coeff = if let Some(ref shift_coeff) = self.shift_coeff {
            format!("{}", shift_coeff)
        } else {
            "NONE".to_owned()
        };
        fmt_report!(fmt, shift_coeff, "shift coefficient (m^-1)");

        fmt_report!(fmt, self.asym_fact, "asymmetry factor");
        Ok(())
    }
}
