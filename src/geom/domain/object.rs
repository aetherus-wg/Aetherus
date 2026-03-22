//! Objects structure that define Surface, Material, SrcId and Attributes

use std::{fmt::Display, path::{Path, PathBuf}};

use aetherus_events::SrcId;
use anyhow::Context;
use mesh_splitting::{Collide, IdxTriangle, Split, mesh::parse_obj, primitives::PrimitiveIdx};
use serde::{Deserialize, Deserializer};

use crate::{
    err::Error, fmt_report, fs::Load, geom::{Mesh, SmoothTriangle, Surface}, math::{Dir3, Point3, Trans3, Trans3Builder}, ord::{Build, Link, Map, Name, Set}, phys::{Material, MaterialBuilder}, sim::{Attribute, AttributeFuture}
};

use mesh_splitting::mesh::{Mesh as IdxMesh};

use log::{debug, info, trace, warn};

/// Object from Wavefront .obj file.
#[derive(Clone)]
pub struct Object {
    /// Scene name it sourced from
    pub scene_name: String,
    /// Object/Surface Name.
    pub obj_name: String,
    /// Material name from .obj file, or going to be populated from mats_map
    pub mat_name: Option<String>,
    /// Resolved material.
    pub mat: Option<Material>,
    /// Mesh built from SmoothTriangles
    pub mesh: IdxMesh,
    /// Source ID used by the UIDs Ledger
    pub src_id: SrcId,
    /// Attribute
    pub attr: Attribute,
}

impl From<IdxTriangle> for SmoothTriangle {
    fn from(tri: IdxTriangle) -> Self {
        let verts = tri.verts.map(|v| Point3::from(v.value));
        let norms = tri.norms.map(|norms| norms.map(|n| Dir3::from(n.value))).unwrap();
        Self::new_from_verts(verts, norms)
    }
}

impl From<IdxMesh> for Mesh {
    fn from(mesh: IdxMesh) -> Self {
        let tris = mesh.tris().into_iter().map(|tri| tri.into()).collect();
        Mesh::new(tris)
    }
}

impl Object {
    pub fn obj_name(&self) -> &str {
        &self.obj_name
    }
    pub fn mat_name(&self) -> Option<&str> {
        self.mat_name.as_deref()
    }
    pub fn get_surface(&self) -> Surface<(Attribute, SrcId)> {
        Surface::new(self.mesh.clone().into(), (self.attr.clone(), self.src_id))
    }

    pub fn with_id(&mut self, src_id: SrcId) -> Result<(), Error> {
        match src_id {
            SrcId::Mat(_) | SrcId::Light(_) => Err(Error::Ledger(format!(
                "Invalid SrcId({}) for Object({}). Expected SrcId::{{MatSurf(_), Surf(_)}}",
                src_id, self.obj_name
            ))),
            SrcId::Surf(_) => {
                self.src_id = src_id;
                Ok(())
            },
            SrcId::MatSurf(_) => {
                // FIXME: Make Attribute reference in_mat the material of the object, such that
                // this src_id assignment doesn't need to be done twice + it will be necessary
                // later on on more complex scenes, where the in_mat becomes out_mat of another object
                self.mat = match &self.mat {
                    Some(mat) => Some(mat.clone().with_id(src_id)),
                    None => {
                        warn!("Material not set for Object({}), cannot assign SrcId::MatSurf from material.", self.obj_name);
                        None
                    },
                };

                self.attr = match &self.attr {
                    Attribute::Interface(in_mat, out_mat) => {
                        Attribute::Interface(in_mat.clone().with_id(src_id), out_mat.clone())
                    },
                    _ => self.attr.clone(),
                };

                self.src_id = src_id.clone();
                Ok(())
            },
            SrcId::None => {
                warn!("Attempting to assign SrcId::None to Object({})", self.obj_name);
                Ok(())
            },
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum AttributeFutureFuture {
    Future(Name),
    Value(AttributeFuture),
}

#[derive(Debug, Clone)]
pub enum ObjFuture {
    Future(PathBuf),
    Value(obj::Obj),
}

impl<'de> Deserialize<'de> for ObjFuture {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let path = PathBuf::deserialize(deserializer)?;
        Ok(ObjFuture::Future(path))
    }
}

pub struct Scene {
    pub name: String,
    objs: obj::Obj,
    transform: Option<Trans3>,
    mats: Set<Material>,
    attrs: Set<AttributeFuture>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SceneBuilder {
    obj: ObjFuture,
    /// Optional transformation.
    transform: Option<Trans3Builder>,
    mats_map: Option<Set<MaterialFuture>>,
    attrs_map: Option<Set<AttributeFutureFuture>>,
}

impl Display for SceneBuilder {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(fmt, "...")?;
        fmt_report!(fmt, match &self.obj {
            ObjFuture::Future(path) => format!("Obj({})", path.display()),
            ObjFuture::Value(obj) => format!("Obj({})", obj.path.display()),
        }, "obj");
        Ok(())
    }
}

impl Load for SceneBuilder {
    type Inst = SceneBuilder;

    fn load(mut self, in_dir: &Path) -> Result<Self::Inst, Error> {
        match &self.obj {
            ObjFuture::Future(path) => {
                let full_path = in_dir.join(path);
                self.obj = ObjFuture::Value(obj::Obj::load(full_path)?);
            }
            ObjFuture::Value(_) => {}
        }
        Ok(self)
    }
}

impl Link<'_, AttributeFuture> for SceneBuilder {
    type Inst = Self;
    fn requires(&self) -> Vec<Name> {
        // FIXME: What is to be added in requires? What is it's purpose?
        todo!()
    }
    fn link(mut self, attrs: &Set<AttributeFuture>) -> Result<Self::Inst, Error> {
        if let Some(attrs_map) = &mut self.attrs_map {
            for name in attrs_map.names_list() {
                let attr_futfut = attrs_map.get_mut(&name).ok_or_else(||
                    Error::Linking(format!("Attributes map missing value of key: {}", name))
                )?;
                *attr_futfut = attr_futfut.clone().link(&attrs)?;
            }
        }
        Ok(self)
    }
}

impl Link<'_, AttributeFuture> for AttributeFutureFuture {
    type Inst = AttributeFutureFuture;
    fn requires(&self) -> Vec<Name> {
        match self {
            AttributeFutureFuture::Future(name) => vec![name.clone()],
            AttributeFutureFuture::Value(_) => vec![],
        }
    }
    fn link(self, attrs: &Set<AttributeFuture>) -> Result<Self::Inst, Error> {
        match self {
            AttributeFutureFuture::Future(name) => {
                let attr_fut = attrs.get(&name)
                    .ok_or_else(||
                        Error::Linking(format!("Attribute {} not found in attributes set during linking.", name))
                    )?;
                Ok(AttributeFutureFuture::Value(attr_fut.clone()))
            },
            AttributeFutureFuture::Value(attr) => Ok(AttributeFutureFuture::Value(attr)),
        }
    }
}

impl Link<'_, Material> for AttributeFutureFuture {
    type Inst = AttributeFutureFuture;
    fn requires(&self) -> Vec<Name> {
        todo!()
    }
    fn link(self, mats: &Set<Material>) -> Result<Self::Inst, Error> {
        match self {
            AttributeFutureFuture::Future(_) => Ok(self),
            AttributeFutureFuture::Value(attr) => {
                Ok(AttributeFutureFuture::Value(attr.link(mats)?))
            },
        }
    }
}

impl Link<'_, Material> for SceneBuilder {
    type Inst = Self;
    fn requires(&self) -> Vec<Name> {
        todo!()
    }
    fn link(mut self, mats: &Set<Material>) -> Result<Self::Inst, Error> {
        // Link in the materials referenced for local use
        if let Some(mats_map) = &mut self.mats_map {
            for name in mats_map.names_list() {
                let mat_fut = mats_map.get_mut(&name).ok_or_else(||
                    Error::Linking(format!("Material map missing value of key: {}", name))
                )?;
                *mat_fut = mat_fut.clone().link(&mats)?;
            }
        }

        // Link materials in the attributes deserialized
        if let Some(attrs_fut) = &mut self.attrs_map {
            *attrs_fut = attrs_fut.clone().link(&mats)?;
        }

        Ok(self)
    }
}

impl Link<'_, Material> for MaterialFuture {
    type Inst = MaterialFuture;
    fn requires(&self) -> Vec<Name> {
        todo!()
    }
    fn link(mut self, mats: &Set<Material>) -> Result<Self::Inst, Error> {
        match self {
            MaterialFuture::Future(MaterialMap::Map(name)) => {
                let mat = mats.get(&name)
                    .ok_or_else(||
                        Error::Linking(format!("Material {} not found in materials set set during linking.", name))
                    )?;
                self = MaterialFuture::Value(mat.clone());
            },
            _ => {}
        }
        Ok(self)
    }
}

impl Build for SceneBuilder {
    type Inst = Scene;
    type MetaInfo = Name;
    fn build(self, id: Self::MetaInfo) -> Result<Self::Inst, Error> {
        let objs = match self.obj {
            ObjFuture::Future(_) => {
                return Err(Error::LoadFile(
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Object not loaded before build.")
                ))
            }
            ObjFuture::Value(obj) => obj,
        };

        let mut mats_map: Map<Name, Material> = Map::new();
        if let Some(mats_future_map) = self.mats_map {
            for (mat_name, mat_future) in mats_future_map {
                match mat_future {
                    MaterialFuture::Future(map) => {
                        match map {
                            MaterialMap::Map(name) => {
                                return Err(Error::Linking(format!("Material linking not implemented yet for {}", name)))
                            }
                            MaterialMap::Builder(builder) => {
                                mats_map.insert(mat_name.clone(), builder.build(mat_name)?);
                            }
                        }
                    }
                    MaterialFuture::Value(mat) => {
                        mats_map.insert(mat_name, mat);
                    }
                }
            }
        }

        // FIXME:: Not necessary to rebuild
        let mut attrs: Map<Name, AttributeFuture> = Map::new();
        if let Some(attrs_future_map) = self.attrs_map {
            for (attr_name, attr_future) in attrs_future_map {
                match attr_future {
                    AttributeFutureFuture::Future(global_attr_name) => {
                        return Err(Error::Linking(
                            format!("Attribute linking not implemented yet for {}", global_attr_name)
                        ));
                    }
                    AttributeFutureFuture::Value(attr) => {
                        attrs.insert(attr_name, attr);
                    }
                }
            }
        }

        let transform = self.transform.map(|transform| transform.build(())).transpose()?;

        let scene = Scene {
            name: id.to_string(),
            objs,
            transform,
            mats: Set::new(mats_map),
            attrs: Set::new(attrs),
        };

        Ok(scene)
    }
}

// FIXME: Cleanup this mess! break down the objects iterator, so it can be more easily understood
// what is happening, especially with error handling collect.
impl Build for Scene {
    type Inst = Vec<Object>;
    type MetaInfo = Name;
    fn build(self, _id: Self::MetaInfo) -> Result<Self::Inst, Error> {

        // FIXME: Figure out how to properly sort out the mismatched version of `obj` dependency.
        // I.e. don't rely on proxy crates data to be consistent across libraries
        let (meshes, verts, norms, faces) = parse_obj(&self.objs.data);

        let mut objects: Vec<Object> = Vec::new();
        let mut resolved_objects = Vec::new();

        for (object_idx, object) in self.objs.data.objects.iter().enumerate() {
            // NOTE: Don't support multiple groups per object
            let obj_name = if object.name == "default" {
                self.name.clone()
            } else {
                format!("{}", object.name)
            };

            let mut mat_name: Option<String> = None;
            for group in &object.groups {
                match &group.material {
                    Some(obj::ObjMaterial::Ref(name)) => {
                        if let Some(ref existing) = mat_name {
                            if *existing != *name {
                                return Err(Error::Linking(format!(
                                    "Multiple material names found for object {}: {} and {}",
                                    obj_name, existing, name
                                )));
                            }
                        } else {
                            mat_name = Some(name.clone());
                        }
                    },
                    None => {},
                    _ => return Err(Error::Linking("Material description from wavefront obj file not supported".to_string())),
                };
            }
            if mat_name.is_none() {
                mat_name = Some(format!("{}_material", object.name.clone()));
            }
            let mat = match &mat_name {
                Some(name) => self.mats.get(&Name::new(&name)).cloned(),
                None => None,
            };

            // TODO: Fallback to object.name maping in materials if a material hasn't been found
            // Attribute::Interface |-> Material is defined
            info!("Building Object: {}, with material: {}", object.name, mat_name.as_deref().unwrap_or("None"));
            let attr = self.attrs
                .get(&Name::new(&object.name))
                .ok_or_else(||
                    std::io::Error::new(std::io::ErrorKind::InvalidData,
                    format!("No Attribute found that matches with obj_name({}) in the attributes map", object.name))
                )?
                .clone()
                .resolve_material(mat.clone())?
                .build(Name::new(&object.name))
                .context(format!("Failed to build Attribute for object {}/{}", self.name, object.name))?;

            objects.push(
                Object {
                    scene_name: self.name.to_string(), // Or use `id` parmeter
                    obj_name,
                    mat_name,
                    mat,
                    mesh: meshes[object_idx].clone(),
                    src_id: SrcId::None,
                    attr,
                });
        }

        info!("Start splitting meshes for Scene: {}", self.name);
        for object in &objects {
            debug!(" > Mesh: {}", object.mesh.name);
        }

        // Splitting of objects at the coplanar intersection with other meshes,
        // to ensure Attribute::Interface is correctly resolved
        let mut idx = objects.len();
        for i in 0..objects.len() {
            for j in i+1..meshes.len() {
                let mesh_i = &objects[i].mesh;
                let mesh_j = &objects[j].mesh;
                if mesh_i.overlap(mesh_j) {
                    info!("Mesh {} and {} overlap", mesh_i.name, mesh_j.name);
                } else {
                    continue;
                }
                let inter = mesh_i.intersect(mesh_j)?;
                trace!("Mesh {} and {} intersection: {:?}", mesh_i.name, mesh_j.name, inter);
                let (new_mesh_i, inter) = mesh_i.split(inter)?;
                let (new_mesh_j, inter) = mesh_j.split(inter)?;
                if !inter.is_empty() {
                    let inter_mesh_name = format!("{}-{}", mesh_i.name, mesh_j.name);
                    let mut inter_mesh = IdxMesh::new(
                        inter_mesh_name,
                        PrimitiveIdx::Global(idx),
                        &verts, &norms, &faces
                    );
                    for tri_inter in inter.into_iter() {
                        // WARN: Information is lost about the normals in the intersection, need to
                        // propagate them from the source triangles
                        inter_mesh.push(tri_inter.into_tris(&mesh_i.faces)?);
                    };
                    resolved_objects.push(
                        Object {
                            scene_name: self.name.clone(),
                            obj_name: format!("{}&{}", objects[i].obj_name, objects[j].obj_name),
                            mat_name: objects[i].mat_name.clone(),
                            mat: objects[i].mat.clone(),
                            mesh: inter_mesh,
                            src_id: objects[i].src_id,
                            attr: Attribute::Interface(objects[i].mat.clone().unwrap(), objects[j].mat.clone().unwrap())
                        }
                    );
                    idx += 1;
                }
                objects[i].mesh = new_mesh_i;
                objects[j].mesh = new_mesh_j;
            }
            resolved_objects.push(objects[i].clone());
        }

        Ok(resolved_objects)
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MaterialMap {
    Map(Name),
    Builder(MaterialBuilder),
}

#[derive(Debug, Clone)]
pub enum MaterialFuture {
    Future(MaterialMap),
    Value(Material),
}

impl<'de> Deserialize<'de> for MaterialFuture {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let builder = MaterialMap::deserialize(deserializer)?;
        Ok(MaterialFuture::Future(builder))
    }
}
