//! Runtime parameters.

use crate::{
    fmt_report,
    geom::{
        object::{Object, Scene, SceneBuilder},
        Boundary, SurfaceLinker, TreeSettings,
    },
    io::output::OutputConfig,
    ord::Set,
    phys::{LightLinker, Material},
    sim::{Engine, LinkerChainStart, Settings},
};
use std::fmt::{Display, Error, Formatter};

/// Runtime parameters.
pub struct Parameters {
    /// Simulation specific settings.
    pub sett: Settings,
    /// Boundary settings.
    pub boundary: Boundary,
    /// Tree settings.
    pub tree: TreeSettings,
    /// Surfaces.
    pub objs: Set<SceneBuilder>,
    /// Attributes.
    pub attrs: Set<LinkerChainStart>,
    /// Materials.
    pub mats: Set<Material>,
    /// Main light.
    pub lights: Set<LightLinker>,
    /// Engine selection.
    pub engine: Engine,
    /// Outputs
    pub output: OutputConfig,
}

impl Parameters {
    /// Construct a new instance.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    #[inline]
    pub const fn new(
        sett: Settings,
        boundary: Boundary,
        tree: TreeSettings,
        objs: Set<SceneBuilder>,
        attrs: Set<LinkerChainStart>,
        mats: Set<Material>,
        lights: Set<LightLinker>,
        engine: Engine,
        output: OutputConfig,
    ) -> Self {
        Self {
            sett,
            boundary,
            tree,
            objs,
            attrs,
            mats,
            lights,
            engine,
            output,
        }
    }
}

impl Display for Parameters {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        writeln!(fmt, "...")?;
        fmt_report!(fmt, self.sett, "settings");
        fmt_report!(fmt, self.boundary, "boundary");
        fmt_report!(fmt, self.tree, "tree settings");
        fmt_report!(fmt, self.objs, "objects");
        fmt_report!(fmt, self.attrs, "attributes");
        fmt_report!(fmt, self.mats, "materials");
        fmt_report!(fmt, self.lights, "lights");
        fmt_report!(fmt, self.engine, "engine");
        fmt_report!(fmt, self.output, "output");
        Ok(())
    }
}
