//! Smooth triangle implementation.

use crate::{
    access,
    err::Error,
    geom::{Collide, Cube, Emit, Ray, Side, Trace, Transformable, Triangle},
    math::{Dir3, Point3, Trans3, Vec3},
    ord::{ALPHA, BETA, GAMMA},
};
use log::{error, warn};
use rand::{Rng, RngExt};

/// Triangle geometry with normal interpolation.
#[derive(Clone, Debug)]
pub struct SmoothTriangle {
    /// Base triangle.
    tri: Triangle,
    /// Normal vectors.
    norms: [Dir3; 3],
}

impl SmoothTriangle {
    access!(tri: Triangle);
    access!(norms: [Dir3; 3]);

    /// Construct a new instance.
    #[must_use]
    pub fn new(tri: Triangle, norms: [Dir3; 3]) -> Self {
        if !norms.iter().all(|&n| n.dot(tri.plane_norm()) > 0.0) {
            warn!("SmoothTriangle normals must point in the same hemisphere direction as the Triangle plane normal\n norms:{:?}, plane norm: {:?}", norms, tri.plane_norm());
        }
        Self { tri, norms }
    }

    /// Construct a new instance from vertices.
    #[must_use]
    pub fn new_from_verts(verts: [Point3; 3], norms: [Dir3; 3]) -> Self {
        Self::new(
            Triangle::new_with_norm(verts, Self::init_plane_norm(&verts, &norms)),
            norms,
        )
    }

    /// Generate SmoothTriangles by interpolating normals for input Triangles using barycentric coordinates
    pub fn partition(&self, tris: Vec<Triangle>) -> Vec<Self> {
        tris.iter().map(|tri| {
            // Find new normal for each vertex based on
            // barycentric coordinates of the original triangle
            let n: Vec<_> = tri.verts().iter().map(|v| {
                if !self.tri.overlap(v) {
                    warn!("Split triangle vertex {:?} is outside the original triangle.", v);
                }
                self.barycentric_coords(v)
                    .map(|(u, v, w)| {
                        Dir3::from(
                            (self.norms[ALPHA] * u) + (self.norms[BETA] * v) + (self.norms[GAMMA] * w),
                        )
                    })
                    .unwrap_or_else(|err| {
                        error!("ERROR: {}.\nFailed to calculate barycentric coordinates for vertex {:?}. Using triangle normal instead.", err, v);
                        *tri.plane_norm()
                    })
            }).collect();
            let n = [n[0], n[1], n[2]];
            SmoothTriangle::new(tri.clone(), n)
        }).collect()
    }

    /// Initialise the plane normal.
    #[must_use]
    fn init_plane_norm(_verts: &[Point3; 3], norms: &[Dir3; 3]) -> Dir3 {
        let n: Vec<_> = norms.iter().map(|&n| n.into_inner()).collect();
        Vec3::from((n[0] + n[1] + n[2]) / 3.0).dir()
    }

    /// Get barycentric_coords of a point in the triangle.
    /// Derived from "Real-Time Collision Detection" - Chapter 3.4
    fn barycentric_coords(&self, point: &Point3) -> Result<(f64, f64, f64), Error> {
        const EPS: f64 = 1e-9;
        let verts = self.tri().verts();
        let v0 = verts[BETA] - verts[ALPHA];
        let v1 = verts[GAMMA] - verts[ALPHA];
        let v2 = *point - verts[ALPHA];
        let d00 = v0.dot(&v0);
        let d01 = v0.dot(&v1);
        let d11 = v1.dot(&v1);
        let d20 = v2.dot(&v0);
        let d21 = v2.dot(&v1);
        let denom = (d00 * d11) - (d01 * d01);
        if denom.abs() < f64::EPSILON {
            // Degenerate triangle (colinear edges), can't calculate barycentric coordinates.
            return Err(Error::Text(
                "Trying to calculate barycentric coordinates for a degenerate triangle."
                    .to_string(),
            ));
        }
        let mut v = ((d11 * d20) - (d01 * d21)) / denom;
        let mut w = ((d00 * d21) - (d01 * d20)) / denom;
        let mut u = 1.0 - v - w;
        if u < -EPS || u > 1.0 + EPS || v < -EPS || v > 1.0 + EPS || w < -EPS || w > 1.0 + EPS {
            // Invalid barycentric coordinates, point outside the triangle
            return Err(Error::Text(format!("Trying to calculate barycentric coordinates ({},{},{}) for a point outside the triangle.", u,v,w)));
        }
        u = u.clamp(0.0, 1.0);
        v = v.clamp(0.0, 1.0);
        w = w.clamp(0.0, 1.0);
        assert!(
            (u + v + w - 1.0).abs() < EPS,
            "Barycentric coordinates do not sum to 1: u={}, v={}, w={}",
            u,
            v,
            w
        );
        assert!(
            u >= 0.0 && u <= 1.0 && v >= 0.0 && v <= 1.0 && w >= 0.0 && w <= 1.0,
            "Barycentric coordinates out of range after clamping: u={}, v={}, w={}",
            u,
            v,
            w
        );
        // Clamp barycentric coordinates to [0,1] to handle numerical precision issues
        Ok((u, v, w))
    }
}

impl From<Triangle> for SmoothTriangle {
    fn from(tri: Triangle) -> Self {
        let norms: [Dir3; 3] = [*tri.plane_norm(), *tri.plane_norm(), *tri.plane_norm()];
        Self::new(tri, norms)
    }
}

impl Collide<Cube> for SmoothTriangle {
    #[inline]
    fn overlap(&self, cube: &Cube) -> bool {
        self.tri.overlap(cube)
    }
}

impl Trace for SmoothTriangle {
    #[inline]
    fn hit(&self, ray: &Ray) -> bool {
        self.tri.hit(ray)
    }

    #[inline]
    fn dist(&self, ray: &Ray) -> Option<f64> {
        self.tri.dist(ray)
    }

    fn dist_side(&self, ray: &Ray) -> Option<(f64, Side)> {
        if let Some((dist, [u, v, w])) = self.tri.intersection_coors(ray) {
            Some((
                dist,
                Side::new(
                    ray.dir(),
                    Dir3::from(
                        (self.norms[BETA] * u) + (self.norms[GAMMA] * v) + (self.norms[ALPHA] * w),
                    ),
                ),
            ))
        } else {
            None
        }
    }
}

impl Transformable for SmoothTriangle {
    fn transform(&mut self, trans: &Trans3) {
        self.tri.transform(trans);

        for n in &mut self.norms {
            *n = Dir3::from(trans.transform_vector(&n.data()));
        }
    }
}

impl Emit for SmoothTriangle {
    fn cast<R: Rng>(&self, rng: &mut R) -> Ray {
        let mut u = rng.random::<f64>();
        let mut v = rng.random::<f64>();

        if (u + v) > 1.0 {
            u = 1.0 - u;
            v = 1.0 - v;
        }
        let w = 1.0 - u - v;

        let edge_a_b = self.tri.verts()[BETA] - self.tri.verts()[ALPHA];
        let edge_a_c = self.tri.verts()[GAMMA] - self.tri.verts()[ALPHA];

        let pos = self.tri.verts()[ALPHA] + (edge_a_b * u) + (edge_a_c * v);
        let dir =
            Dir3::from((self.norms[BETA] * u) + (self.norms[GAMMA] * v) + (self.norms[ALPHA] * w));

        Ray::new(pos, dir)
    }
}

impl PartialEq for SmoothTriangle {
    fn eq(&self, other: &Self) -> bool {
        self.tri.verts() == other.tri.verts()
    }
}
impl Eq for SmoothTriangle {}

impl Collide<Triangle> for SmoothTriangle {
    fn overlap(&self, other: &Triangle) -> bool {
        self.tri.overlap(other)
    }
}

impl Collide<SmoothTriangle> for SmoothTriangle {
    fn overlap(&self, other: &SmoothTriangle) -> bool {
        self.tri.overlap(&other.tri)
    }
}
