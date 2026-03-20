//! Mesh loader.

use crate::{
    err::Error,
    fs::{File, Load},
    geom::{Mesh, Transformable},
    math::Trans3Builder,
    ord::Build,
};
use arctk_attr::file;
use std::path::{Path, PathBuf};

// TODO: Do we really need to load a list of object files?
// It would be more useful to do the oposite, of getting multiple objects/meshes
// from a single obj file as they are normally exported from Blender

/// Loadable triangle mesh conglomerate structure.
#[file]
pub struct MeshLoader(
    /// Wavefront object file.
    PathBuf,
    /// Optional transformation.
    Option<Trans3Builder>,
);

impl Load for MeshLoader {
    type Inst = Mesh;

    fn load(self, in_dir: &Path) -> Result<Self::Inst, Error> {
        let trans = self.1.map(|trans3_builder| trans3_builder.build(())).transpose()?;

        let mut tris = Vec::new();
        let mut obj = Self::Inst::new_from_file(&in_dir.join(self.0))?;
        if let Some(t) = trans {
            obj.transform(&t);
        }
        tris.extend(obj.into_tris());

        Ok(Self::Inst::new(tris))
    }
}
