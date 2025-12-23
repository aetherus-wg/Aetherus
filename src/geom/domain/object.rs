//! Objects structure that define Surface, Material, SrcId and Attributes

use std::{fmt::Display, path::{Path, PathBuf}, rc::Rc};

use aetherus_events::mcrt::SrcId;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    err::Error, fmt_report, fs::Load, geom::{Mesh, SmoothTriangle, Surface, Transformable}, math::{Dir3, Point3, Trans3, Trans3Builder}, ord::{Build, Link, Map, Name, Set}, phys::{Material, MaterialBuilder}, sim::{Attribute, AttributeFuture}
};

use log::warn;

/// Object from Wavefront .obj file.
#[derive(Clone)]
pub struct Object {
    /// Object/Surface Name.
    pub obj_name: String,
    /// Material name from .obj file, or going to be populated from mats_map
    pub mat_name: Option<String>,
    /// Resolved material.
    pub mat: Option<Material>,
    /// Mesh built from SmoothTriangles
    pub mesh: Mesh,
    /// Source ID used by the UIDs Ledger
    pub src_id: SrcId,
    /// Attribute
    pub attr: Attribute,
}

impl Object {
    pub fn empty() -> Self {
        Self {
            obj_name: String::new(),
            mat_name: None,
            mat: None,
            mesh: Mesh::new(vec![]),
            src_id: SrcId::MatSurf(0),
            attr: Attribute::Mirror(0.0),
        }
    }
    pub fn obj_name(&self) -> &str {
        &self.obj_name
    }
    pub fn mat_name(&self) -> Option<&str> {
        self.mat_name.as_deref()
    }
    pub fn get_surface(&self) -> Surface<'_,Object> {
        Surface::new(self.mesh.clone(), self)
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

#[derive(Debug)]
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
    name: String,
    objs: obj::Obj,
    transform: Option<Trans3>,
    mats_map: Set<Material>,
    attrs_map: Set<AttributeFuture>,
}

#[derive(Deserialize, Debug)]
pub struct SceneBuilder {
    obj: ObjFuture,
    /// Optional transformation.
    transform: Option<Trans3Builder>,
    mats_map: Option<Set<MaterialFuture>>,
    attrs_map: Option<Set<AttributeFutureFuture>>,
}

impl Display for SceneBuilder {
    #[inline]
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

    #[inline]
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

impl Link<'_, Material> for SceneBuilder {
    type Inst = Self;
    fn requires(&self) -> Vec<Name> {
        todo!()
    }
    fn link(mut self, mats: &Set<Material>) -> Result<Self::Inst, Error> {
        if let Some(mats_map) = &mut self.mats_map {
            for name in mats_map.names_list() {
                let mat_fut = mats_map.get_mut(&name).ok_or_else(||
                    Error::Linking(format!("Material map missing value of key: {}", name))
                )?;
                *mat_fut = mat_fut.clone().link(&mats)?;
            }
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

impl Build for Set<SceneBuilder> {
    type Inst = Vec<Scene>;
    fn build(self) -> Result<Self::Inst, Error> {
        let mut scenes = Vec::with_capacity(self.len());
        for (scene_name, builder) in self {
            // FIXME: Handle errors properly
            let objs = match &builder.obj {
                ObjFuture::Future(_) => {
                    return Err(Error::LoadFile(
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Object not loaded before build.")
                    ))
                }
                ObjFuture::Value(obj) => obj.clone(),
            };
            let mut mats_map: Map<Name, Material> = Map::new();
            let mut attrs_map: Map<Name, AttributeFuture> = Map::new();

            if let Some(mats_future_map) = builder.mats_map {
                for (mat_name, mat_future) in mats_future_map {
                    match mat_future {
                        MaterialFuture::Future(map) => {
                            match map {
                                MaterialMap::Map(name) => {
                                    return Err(Error::Linking(format!("Material linking not implemented yet for {}", name)))
                                }
                                MaterialMap::Builder(builder) => {
                                    mats_map.insert(mat_name, builder.build()?);
                                }
                            }
                        }
                        MaterialFuture::Value(mat) => {
                            mats_map.insert(mat_name, mat);
                        }
                    }
                }
            }

            if let Some(attrs_future_map) = builder.attrs_map {
                for (attr_name, attr_future) in attrs_future_map {
                    match attr_future {
                        AttributeFutureFuture::Future(_) => {
                            return Err(Error::Linking("Attribute linking not implemented yet".to_string()))
                        }
                        AttributeFutureFuture::Value(attr) => {
                            attrs_map.insert(attr_name, attr);
                        }
                    }
                }
            }

            let transform = builder.transform.map(Build::build).transpose()?;

            let scene = Scene {
                name: scene_name.to_string(),
                objs,
                transform,
                mats_map: Set::new(mats_map),
                attrs_map: Set::new(attrs_map),
            };
            scenes.push(scene);

        }
        Ok(scenes)
    }
}

// FIXME: Cleanup this mess! break down the objects iterator, so it can be more easily understood
// what is happening, especially with error handling collect.
impl Build for Scene {
    type Inst = Vec<Object>;
    fn build(self) -> Result<Self::Inst, Error> {
        let attrs = self.attrs_map.clone().build()?;

        let verts = self.objs.data.position.iter()
            .map(|vs| {
                let vs: Vec<f64> = vs.iter().map(|v| *v as f64).collect();
                Point3::new(vs[0], vs[1], vs[2])
            })
            .collect::<Vec<_>>();

        let norms = self.objs.data.normal.iter()
            .map(|vs| {
                let vs: Vec<f64> = vs.iter().map(|v| *v as f64).collect();
                Dir3::new(vs[0], vs[1], vs[2])
            })
            .collect::<Vec<_>>();

        self.objs.data.objects
        .iter()
        .map(|obj| {
            // NOTE: Don't support multiple groups per object
            let faces_idx = obj.groups
                .iter()
                .flat_map(|group| {
                    group.polys.iter().map(|poly| {
                        if poly.0.len() != 3 {
                            return Err("Only triangular faces are supported.");
                            // TODO: Could easily convert a quadrilaterl into 2 triangles, hence supporting
                            // more export options for OBJ files
                        }

                        let v_idx: Vec<_> = poly.0.iter()
                            .map(|idx_tuple| idx_tuple.0)
                            .collect();
                        //let t_idx = poly.0.map(|idx_tuple| idx_tuple.1);
                        let n_idx: Vec<_> = poly.0.iter()
                            .map(|idx_tuple| idx_tuple.2.ok_or("Missing normal index."))
                            .collect::<Result<Vec<_>, _>>()?;
                        Ok(((v_idx[0], v_idx[1], v_idx[2]), (n_idx[0], n_idx[1], n_idx[2])))
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;

            let tris = faces_idx.into_iter()
                .map(|face|
                    SmoothTriangle::new_from_verts(
                        [verts[(face.0).0], verts[(face.0).1], verts[(face.0).2]],
                        [norms[(face.1).0], norms[(face.1).1], norms[(face.1).2]],
                    ))
                .collect();
            let mut mesh = Mesh::new(tris);
            if let Some(t) = self.transform {
                mesh.transform(&t);
            }

            let obj_name = if obj.name == "default" {
                self.name.clone()
            } else {
                format!("{}/{}", self.name, obj.name)
            };

            let mat_name = match &obj.groups[0].material {
                Some(obj::ObjMaterial::Ref(name)) => Some(name.clone()),
                None => Some(obj.name.clone()),
                _ => return Err(Error::Linking("Material description from wavefront obj file not supported".to_string())),
            };

            let mat = match &mat_name {
                Some(name) => self.mats_map.get(&Name::new(&name)).cloned(),
                None => None,
            };

            let attr = attrs.get(&Name::new(&obj.name))
                            .ok_or_else(||
                                std::io::Error::new(std::io::ErrorKind::InvalidData,
                                format!("No Attribute found that matches with obj_name({}) in the attributes map", obj.name))
                            )?.clone();

            Ok(
                Object {
                    obj_name,
                    mat_name,
                    mat,
                    mesh,
                    src_id: SrcId::MatSurf(0),
                    attr,
                }
            )
        })
        .collect::<Result<Vec<_>,_>>()
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
