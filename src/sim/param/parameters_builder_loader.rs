//! Loadable parameters.

use crate::{
    err::Error, fs::{Load, Redirect}, geom::{boundary_builder::BoundaryBuilder, SurfaceLinkerLoader, TreeSettings}, io::output::OutputConfig, ord::Set, phys::{LightLinkerBuilderLoader, MaterialBuilder}, sim::{
        AttributeLinkerLinkerLinkerLinkerLinkerLinker, EngineBuilderLoader, ParametersBuilder, Settings,
    }
};
use arctk_attr::file;
use std::path::Path;

/// Loadable runtime parameters.
#[file]
pub struct ParametersBuilderLoader {
    /// Simulation specific settings.
    sett: Redirect<Settings>,
    // Boundary conditions. 
    boundary: Redirect<BoundaryBuilder>,
    /// Tree settings.
    tree: Redirect<TreeSettings>,
    /// Surfaces.
    surfs: Redirect<Set<SurfaceLinkerLoader>>,
    /// Attributes.
    attrs: Redirect<Set<AttributeLinkerLinkerLinkerLinkerLinkerLinker>>,
    /// Materials.
    mats: Redirect<Set<Redirect<MaterialBuilder>>>,
    /// Main light.
    lights: Redirect<Set<LightLinkerBuilderLoader>>,
    /// Engine selection.
    engine: EngineBuilderLoader,
    /// Output
    output: Redirect<OutputConfig>,
}

impl Load for ParametersBuilderLoader {
    type Inst = ParametersBuilder;

    #[inline]
    fn load(self, in_dir: &Path) -> Result<Self::Inst, Error> {
        let sett = self.sett.load(in_dir)?;
        let boundary = self.boundary.load(in_dir)?;
        let tree = self.tree.load(in_dir)?;
        let surfs = self.surfs.load(in_dir)?.load(in_dir)?;
        let attrs = self.attrs.load(in_dir)?;
        let mats = self.mats.load(in_dir)?.load(in_dir)?;
        let lights = self.lights.load(in_dir)?.load(in_dir)?;
        let engine = self.engine.load(in_dir)?;
        let output = self.output.load(in_dir)?;

        Ok(Self::Inst::new(
            sett, boundary, tree, surfs, attrs, mats, lights, engine, output,
        ))
    }
}
