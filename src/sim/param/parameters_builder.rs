//! Buildable parameters.

use crate::{
    err::Error,
    fmt_report,
    geom::{object::SceneBuilder, BoundaryBuilder, SurfaceLinker, TreeSettings},
    io::output::OutputConfig,
    ord::{Build, Set},
    phys::{LightLinkerBuilder, MaterialBuilder},
    sim::{EngineBuilder, LinkerChainStart, Parameters, Settings},
};
use std::fmt::{Display, Formatter};

/// Buildable runtime parameters.
pub struct ParametersBuilder {
    /// Simulation specific settings.
    sett: Settings,
    /// Boundary settings.
    boundary: BoundaryBuilder,
    /// Tree settings.
    tree: TreeSettings,
    /// Objects.
    objs: Set<SceneBuilder>,
    /// Attributes.
    attrs: Set<LinkerChainStart>,
    /// Materials.
    mats: Set<MaterialBuilder>,
    /// Main light.
    lights: Set<LightLinkerBuilder>,
    /// Engine selection.
    engine: EngineBuilder,
    /// Output
    output: OutputConfig,
}

impl ParametersBuilder {
    /// Construct a new instance.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    #[inline]
    pub const fn new(
        sett: Settings,
        boundary: BoundaryBuilder,
        tree: TreeSettings,
        objs: Set<SceneBuilder>,
        attrs: Set<LinkerChainStart>,
        mats: Set<MaterialBuilder>,
        lights: Set<LightLinkerBuilder>,
        engine: EngineBuilder,
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

impl Build for ParametersBuilder {
    type Inst = Parameters;

    #[inline]
    fn build(self) -> Result<Self::Inst, Error> {
        let sett = self.sett;
        let boundary = self.boundary.build();
        let tree = self.tree;
        let objs = self.objs;
        let attrs = self.attrs;
        let mats = self.mats.build()?;
        let light = self.lights.build()?;
        let engine = self.engine.build()?;
        let output = self.output;

        Ok(Self::Inst::new(
            sett, boundary, tree, objs, attrs, mats, light, engine, output,
        ))
    }
}

impl Display for ParametersBuilder {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
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
