//! Smooth triangle-mesh implementation.

use crate::{
    access, clone, err::Error, fmt_report, fs::{File, mesh_from_objfile, mesh_from_ugrid}, geom::{Collide, Cube, Emit, Ray, Side, SmoothTriangle, Split, Trace, Transformable}, math::Trans3, ord::{ALPHA, cartesian::X}
};
use log::{debug, info};
use rand::{Rng, RngExt};
use std::{
    collections::VecDeque, fmt::{Display, Formatter}, path::Path
};
use anyhow::Context;

/// Boundary padding.
const PADDING: f64 = 1e-6;

/// Mesh geometry.
#[derive(Clone)]
pub struct Mesh {
    /// Bounding box.
    boundary: Cube,
    /// List of component triangles.
    tris: Vec<SmoothTriangle>,
    /// Total surface area.
    area: f64,
}

impl Mesh {
    access!(boundary: Cube);
    access!(tris: Vec<SmoothTriangle>);
    clone!(area: f64);

    /// Construct a new instance.
    #[must_use]
    pub fn new(tris: Vec<SmoothTriangle>) -> Self {
        let area = tris.iter().map(|tri| tri.tri().squared_area().sqrt()).sum();

        Self {
            boundary: Self::init_boundary(&tris),
            tris,
            area,
        }
    }

    /// Initialise the bounding box for the mesh.
    #[must_use]
    fn init_boundary(tris: &[SmoothTriangle]) -> Cube {
        let mut mins = tris[X].tri().verts()[ALPHA];
        let mut maxs = mins;

        for tri in tris {
            for v in tri.tri().verts().iter() {
                for (v, (min, max)) in v.iter().zip(mins.iter_mut().zip(maxs.iter_mut())) {
                    if *min > *v {
                        *min = *v;
                    } else if *max < *v {
                        *max = *v;
                    }
                }
            }
        }

        for (max, min) in maxs.iter_mut().zip(mins.iter_mut()) {
            *min -= PADDING;
            *max += PADDING;
        }

        Cube::new(mins, maxs)
    }

    /// Destruct the instance and retrieve the list of triangles.
    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn into_tris(self) -> Vec<SmoothTriangle> {
        self.tris
    }
}

impl Collide<Cube> for Mesh {
    #[inline]
    fn overlap(&self, cube: &Cube) -> bool {
        if !self.boundary.overlap(cube) {
            return false;
        }

        for tri in &self.tris {
            if tri.overlap(cube) {
                return true;
            }
        }

        false
    }
}

impl Transformable for Mesh {
    #[inline]
    fn transform(&mut self, trans: &Trans3) {
        for tri in &mut self.tris {
            tri.transform(trans);
        }

        self.boundary = Self::init_boundary(&self.tris);
    }
}

impl Emit for Mesh {
    fn cast<R: Rng>(&self, rng: &mut R) -> Ray {
        let r = rng.random_range(0.0..self.area);
        let mut total_area = 0.0;
        for tri in &self.tris {
            total_area += tri.tri().squared_area().sqrt();
            if total_area >= r {
                return tri.cast(rng);
            }
        }

        unreachable!()
    }
}

impl Trace for Mesh {
    #[inline]
    fn hit(&self, ray: &Ray) -> bool {
        if !self.boundary.hit(ray) {
            return false;
        }

        self.tris.iter().any(|t| t.hit(ray))
    }

    #[inline]
    fn dist(&self, ray: &Ray) -> Option<f64> {
        if !self.boundary.hit(ray) {
            return None;
        }

        self.tris
            .iter()
            .filter_map(|tri| tri.dist(ray))
            .min_by(|a, b| a.partial_cmp(b).unwrap())
    }

    #[inline]
    fn dist_side(&self, ray: &Ray) -> Option<(f64, Side)> {
        if !self.boundary.hit(ray) {
            return None;
        }

        self.tris
            .iter()
            .filter_map(|tri| tri.dist_side(ray))
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
    }
}

impl Display for Mesh {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        writeln!(fmt, "...")?;
        fmt_report!(fmt, self.boundary, "boundary");
        fmt_report!(fmt, self.tris.len(), "num triangles");
        fmt_report!(fmt, self.area, "area (m)");
        Ok(())
    }
}

impl File for Mesh {
    fn load(path: &Path) -> Result<Self, Error> {
        if path.extension().unwrap() == "obj" {
            let mesh_tris = mesh_from_objfile(path)
                .context(format!("Unable to read mesh from wavefront file: {}", path.display()))?;

            Ok(Self::new(mesh_tris))

        } else if path.extension().unwrap() == "nc" {
            let mesh_tris = mesh_from_ugrid(path).unwrap_or_else(|_| {
                panic!("Unable to read mesh from ugrid file: {}", path.display())
            });

            Ok(Self::new(mesh_tris))

        } else {
            panic!("Mesh file {} has unsupported file type", path.display());
        }

    }
}

impl Collide<Mesh> for Mesh {
    fn overlap(&self, other: &Mesh) -> bool {
        if !self.boundary.overlap(other.boundary()) {
            return false;
        }
        for tri in &self.tris {
            for other_tri in other.tris() {
                if tri.overlap(other_tri) {
                    return true;
                }
            }
        }
        false
    }
}

impl Split<Mesh, Mesh> for Mesh {
    type Inst = Mesh;
    fn split_transparent(&self, other: &Mesh) -> (Self::Inst, Mesh) {
        let mut u_tris: VecDeque<_> = self.tris().iter().map(|tri| tri.clone()).collect();
        let mut v_tris: Vec<_> = other.tris().iter().map(|tri| tri.clone()).collect();

        let mut final_u_tris = Vec::new();

        while !u_tris.is_empty()
        {
            let u_tri = u_tris.pop_front().unwrap();
            let mut v_tris_mutated = Vec::new();
            let mut new_surf_v_tris = Vec::new();
            let mut u_tri_collision = false;

            debug!("Checking for collisions of {:?} to {} triangles", u_tri, v_tris.len());
            for (v_idx, v_tri) in v_tris.iter().enumerate() {
                // FIXME: Triangle overlap should exclude triangles that just share an edge or vertex
                if u_tri.overlap(v_tri) {
                    let (new_u_tris, split_segs) = u_tri.split_transparent(&v_tris[v_idx]);
                    if new_u_tris.len() > 1 {
                        debug!("Splitting triangle {:?}: into {} triangles.", u_tri, new_u_tris.len());
                        new_u_tris
                            .iter()
                            .for_each(|new_u_tri|
                                u_tris.push_back(new_u_tri.clone())
                            );
                        u_tri_collision = true;
                    }
                    let new_v_tris = v_tris[v_idx].split(&split_segs);
                    if new_v_tris.len() > 1 {
                        debug!("Splitting triangle {:?}:{} into {} triangles.", v_tris[v_idx], v_idx, new_v_tris.len());
                        v_tris_mutated.push(v_idx);
                        new_v_tris
                            .iter()
                            .for_each(|new_v_tri|
                                new_surf_v_tris.push(new_v_tri.clone())
                            );
                    }
                    assert!(new_u_tris.len() > 0 && new_v_tris.len() > 0, "Not a valid triangle collision occured");
                    if u_tri_collision {
                        info!("Overlapping surfaces");
                        debug!("Overlapping surfaces between: {:?} and {:?}", u_tri, v_tri);
                        break;
                    }
                }
            }

            if !u_tri_collision {
                final_u_tris.push(u_tri);
            }

            let mut offset = 0;
            for v_idx in v_tris_mutated {
                v_tris.remove(v_idx - offset);
                offset += 1;
            }
            v_tris.extend(new_surf_v_tris);
        }

        (Mesh::new(final_u_tris), Mesh::new(v_tris))
    }
    fn split(&self, other: &Mesh) -> Self::Inst {
        self.split_transparent(other).0
    }
}
