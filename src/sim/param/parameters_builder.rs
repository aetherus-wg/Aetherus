//! Buildable parameters.

use crate::{
    fmt_report, geom::{BoundaryBuilder, SurfaceLinker, TreeSettings}, io::output::OutputConfig, ord::{Build, Set}, phys::{LightLinkerBuilder, MaterialBuilder}, sim::{LinkerChainStart, EngineBuilder, Parameters, Settings}
};
use std::fmt::{Display, Error, Formatter};

/// Buildable runtime parameters.
pub struct ParametersBuilder {
    /// Simulation specific settings.
    sett: Settings,
    /// Boundary settings. 
    boundary: BoundaryBuilder,
    /// Tree settings.
    tree: TreeSettings,
    /// Surfaces.
    surfs: Set<SurfaceLinker>,
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
        surfs: Set<SurfaceLinker>,
        attrs: Set<LinkerChainStart>,
        mats: Set<MaterialBuilder>,
        lights: Set<LightLinkerBuilder>,
        engine: EngineBuilder,
        output: OutputConfig
    ) -> Self {
        Self {
            sett,
            boundary,
            tree,
            surfs,
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
    fn build(self) -> Self::Inst {
        let sett = self.sett;
        let boundary = self.boundary.build();
        let tree = self.tree;
        let surfs = self.surfs;
        let attrs = self.attrs;
        let mats = self.mats.build();
        let light = self.lights.build();
        let engine = self.engine.build();
        let output = self.output;

        Self::Inst::new(sett, boundary, tree, surfs, attrs, mats, light, engine, output)
    }
}

impl Display for ParametersBuilder {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        writeln!(fmt, "...")?;
        fmt_report!(fmt, self.sett, "settings");
        fmt_report!(fmt, self.boundary, "boundary");
        fmt_report!(fmt, self.tree, "tree settings");
        fmt_report!(fmt, self.surfs, "surfaces");
        fmt_report!(fmt, self.attrs, "attributes");
        fmt_report!(fmt, self.mats, "materials");
        fmt_report!(fmt, self.lights, "lights");
        fmt_report!(fmt, self.engine, "engine");
        fmt_report!(fmt, self.output, "output");
        Ok(())
    }
}
