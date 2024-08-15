//! Simulation input.

use crate::{
    fmt_report,
    geom::{Boundary, Tree},
    ord::{Register, Set},
    phys::{Light, Material},
    sim::{Attribute, Settings},
};
use std::fmt::{Display, Error, Formatter};

/// MCRT simulation resources conglomerate.
#[derive(Clone)]
pub struct Input<'a> {
    /// Spectrometer register.
    pub spec_reg: &'a Register,
    /// Materials.
    pub mats: &'a Set<Material>,
    /// Attributes.
    pub attrs: &'a Set<Attribute<'a>>,
    /// Emission light.
    pub light: Light<'a>,
    /// Hit-scan tree.
    pub tree: &'a Tree<'a, Attribute<'a>>,
    /// General settings.
    pub sett: &'a Settings,
    /// Boundary for the simulation. 
    pub bound: &'a Boundary,
}

impl<'a> Input<'a> {
    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub const fn new(
        spec_reg: &'a Register,
        mats: &'a Set<Material>,
        attrs: &'a Set<Attribute>,
        light: Light<'a>,
        tree: &'a Tree<Attribute>,
        sett: &'a Settings,
        bound: &'a Boundary
    ) -> Self {
        Self {
            spec_reg,
            mats,
            attrs,
            light,
            tree,
            sett,
            bound,
        }
    }
}

impl Display for Input<'_> {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        writeln!(fmt, "...")?;
        fmt_report!(fmt, self.spec_reg, "spectrometer register");
        fmt_report!(fmt, self.attrs, "materials");
        fmt_report!(fmt, self.attrs, "attributes");
        fmt_report!(fmt, self.light, "light");
        fmt_report!(fmt, self.tree, "hit-scan tree");
        fmt_report!(fmt, self.sett, "settings");
        fmt_report!(fmt, self.bound, "boundary");
        Ok(())
    }
}
