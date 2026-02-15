//! Flat Triangle
//!
//! This module provides `Triangle`---an implementation of a flat triangle.
//! As an an example, a new instance of an isoceles triangle can be created using:
//! ```rust
//! # use aetherus::geom::Triangle;
//! # use aetherus::math::Point3;
//! let tri = Triangle::new([Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0), Point3::new(0.5, 1.0, 0.0)]);
//!
//! // The geometric properties of this triangle can be interrogated
//! // Perimeter.
//! println!("{}", tri.perimeter())
//! ```

use core::f64;
use std::usize;

use crate::{
    access,
    geom::{Collide, Cube, Emit, Ray, Side, Trace, Transformable, segment::Segment},
    math::{Dir3, Point3, Trans3, Vec3},
    ord::{ALPHA, BETA, GAMMA},
};
use log::warn;
use rand::{Rng, RngExt};


/// Triangle.
#[derive(Clone, Debug, PartialEq)]
pub struct Triangle {
    /// Vertex points.
    verts: [Point3; 3],
    /// Surface plane normal.
    plane_norm: Dir3,
}

impl Triangle {
    access!(verts: [Point3; 3]);
    access!(plane_norm: Dir3);

    /// Construct a new instance.
    #[must_use]
    pub fn new(verts: [Point3; 3]) -> Self {
        let plane_norm = Self::init_plane_norm(&verts);

        Self { verts, plane_norm }
    }

    /// Initialise the plane normal.
    #[must_use]
    fn init_plane_norm(verts: &[Point3; 3]) -> Dir3 {
        Dir3::from((verts[ALPHA] - verts[GAMMA]).cross(&(verts[BETA] - verts[ALPHA])))
    }

    /// Calculate the side lengths.
    #[inline]
    #[must_use]
    pub fn side_lengths(&self) -> [f64; 3] {
        let ab = nalgebra::distance(&self.verts[ALPHA].data(), &self.verts[BETA].data());
        let bc = nalgebra::distance(&self.verts[BETA].data(), &self.verts[GAMMA].data());
        let ca = nalgebra::distance(&self.verts[GAMMA].data(), &self.verts[ALPHA].data());

        [ab, bc, ca]
    }

    /// Calculate the perimeter length.
    #[inline]
    #[must_use]
    pub fn perimeter(&self) -> f64 {
        let [ab, bc, ca] = self.side_lengths();
        ab + bc + ca
    }

    /// Calculate the surface area.
    #[inline]
    #[must_use]
    pub fn squared_area(&self) -> f64 {
        let [ab, bc, ca] = self.side_lengths();
        let s = (ab + bc + ca) * 0.5;
        s * (s - ab) * (s - bc) * (s - ca)
    }

    /// Centre point.
    #[inline]
    #[must_use]
    pub fn centre(&self) -> Point3 {
        ((self.verts[ALPHA].to_homogeneous()
            + self.verts[BETA].to_homogeneous()
            + self.verts[GAMMA].to_homogeneous())
            / 3.0)
            .xyz()
    }

    pub fn aabb(&self) -> Cube {
        let mins = self.verts
            .iter()
            .fold(Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY), |acc, v| {
                Point3::new(
                    acc.x().min(v.x()),
                    acc.y().min(v.y()),
                    acc.z().min(v.z()),
                )
        });
        let maxs = self.verts
            .iter()
            .fold(Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY), |acc, v| {
                Point3::new(
                    acc.x().max(v.x()),
                    acc.y().max(v.y()),
                    acc.z().max(v.z()),
                )
        });
        Cube::new(mins, maxs)
    }

    /// Determine the intersection distance along a `Ray`'s direction.
    /// Also return the barycentric intersection coordinates.
    #[must_use]
    pub fn intersection_coors(&self, ray: &Ray) -> Option<(f64, [f64; 3])> {
        let verts = self.verts;

        let e1 = verts[BETA] - verts[ALPHA];
        let e2 = verts[GAMMA] - verts[ALPHA];

        let d_cross_e2 = ray.dir().cross_vec(&e2);
        let e1_dot_d_cross_e2 = e1.dot(&d_cross_e2);

        if e1_dot_d_cross_e2.abs() <= 0.0 {
            return None;
        }

        let inv_e1_dot_d_cross_e2 = 1.0 / e1_dot_d_cross_e2;
        let rel_pos = ray.pos() - verts[ALPHA];
        let u = inv_e1_dot_d_cross_e2 * rel_pos.dot(&d_cross_e2);

        if !(0.0..=1.0).contains(&u) {
            return None;
        }

        let q = rel_pos.cross(&e1);
        let v = inv_e1_dot_d_cross_e2 * ray.dir().dot_vec(&q);

        if (v < 0.0) || ((u + v) > 1.0) {
            return None;
        }

        let dist = inv_e1_dot_d_cross_e2 * e2.dot(&q);

        if dist <= 0.0 {
            return None;
        }

        let w = 1.0 - (u + v);

        Some((dist, [u, v, w]))
    }


    pub fn vertex_in(&self, vertex: Point3) -> bool {
        let segs = [
            Segment::new(self.verts[ALPHA], self.verts[BETA]),
            Segment::new(self.verts[BETA], self.verts[GAMMA]),
            Segment::new(self.verts[GAMMA], self.verts[ALPHA]),
        ];
        let mut sign: Option<bool> = None;

        // 1. First check that vertex is in the plane of the triangle
        let vertex_to_tri = vertex - self.verts[ALPHA];
        if self.plane_norm.dot_vec(&vertex_to_tri).abs() > 1e-9 {
            return false;
        }

        // 2. Then check that vertex is on the same side of all edges of the triangle
        for seg in &segs {
            let edge_vec = seg.end - seg.start;
            let to_vertex_vec = vertex - seg.start;
            let cross_prod = edge_vec.cross(&to_vertex_vec);
            // 2. Check if vertex is on an edge of the triangle
            if cross_prod.abs() < 1e-9 {
                return false;
            }
            // 3. Check that the vertex is on the same side of each segment
            let dir = self.plane_norm.dot_vec(&cross_prod) > 0.0;
            match sign {
                None => {
                        sign = Some(dir);
                }
                Some(sign) => {
                    if sign != dir {
                        return false;
                    }
                }
            };
        }
        true
    }

    // TODO: Test scenario when vertices overlap
    pub fn triangle_split(&self, other: &Triangle) -> Vec<Triangle> {

        #[derive(Debug)]
        struct SegList {
            segs: Vec<(usize, usize)>,
        }

        impl SegList {
            fn new() -> Self {
                Self { segs: Vec::new() }
            }
            fn push(&mut self, seg: (usize, usize)) {
                let new_seg = if seg.0 < seg.1 { seg } else { (seg.1, seg.0) };
                if !self.segs.contains(&new_seg) {
                    self.segs.push(new_seg);
                }
            }
            fn inner(&self) -> &Vec<(usize, usize)> {
                &self.segs
            }
        }

        #[derive(Debug)]
        struct TriList {
            tris: Vec<(usize, usize, usize)>,
        }

        impl TriList {
            fn new() -> Self {
                Self { tris: Vec::new() }
            }
            fn push(&mut self, tri: (usize, usize, usize)) {
                let mut tri_vec = vec![tri.0, tri.1, tri.2];
                tri_vec.sort_unstable();
                let new_tri = (tri_vec[0], tri_vec[1], tri_vec[2]);
                if !self.tris.contains(&new_tri) {
                    self.tris.push(new_tri);
                }
            }
            fn inner(&self) -> &Vec<(usize, usize, usize)> {
                &self.tris
            }
        }

        let mut verts = vec![
            self.verts[ALPHA],
            self.verts[BETA],
            self.verts[GAMMA],
        ];

        let segs_v: [(usize, usize); 3] = [(0, 1), (1, 2), (2, 0)];

        let mut new_segs = SegList::new();

        let mut verts_in = vec![];
        let mut verts_out = vec![];
        let mut segs_u = vec![];

        // Resolve vertices and segments inside the triangle
        for &v in &other.verts {
            if self.vertex_in(v) {
                verts_in.push(verts.len());
                verts.push(v);
            } else {
                verts_out.push(v);
            }
        }
        for i in 0..verts_in.len() {
            for j in (i+1)..verts_in.len() {
                new_segs.push((verts_in[i], verts_in[j]));
            }
        }
        println!("Verts in: {:?}, Verts out: {:?}", verts_in, verts_out);
        match verts_out.len() {
            0 => {
                assert_eq!(verts_in.len(), 3);
                // No vertices outside. The other triangle is fully contained in this one.
            }
            1 => {
                assert_eq!(verts_in.len(), 2);
                segs_u.push((Some(verts_in[0]), Segment::new(verts_out[0], verts[verts_in[0]])));
                segs_u.push((Some(verts_in[1]), Segment::new(verts_out[0], verts[verts_in[1]])));
            }
            2 => {
                assert_eq!(verts_in.len(), 1);
                segs_u.push((Some(verts_in[0]), Segment::new(verts_out[0], verts[verts_in[0]])));
                segs_u.push((Some(verts_in[0]), Segment::new(verts_out[1], verts[verts_in[0]])));
            }
            3 => {
                assert_eq!(verts_in.len(), 0);
                segs_u.push((None, Segment::new(verts_out[0], verts_out[1])));
                segs_u.push((None, Segment::new(verts_out[1], verts_out[2])));
                segs_u.push((None, Segment::new(verts_out[2], verts_out[0])));
            }
            _ => unreachable!(),
        }

        println!("Segments in triangle: {:?}", segs_u);


        // Find all intersections and segments that need to be included in the new triangles
        #[derive(Debug)]
        struct SegWithIntersection {
            start: usize,
            end: usize,
            inter_idx: usize,
        }


        let mut inters_idx = vec![];

        for (vertex_u, seg_u) in segs_u {
            println!("Vertex {:?} is inside the triangle. Segment: {:?}", vertex_u, seg_u);
            let mut segs_inter = Vec::new();
            let intersections: Vec<_> = segs_v.iter()
                // TODO: Check if order of interesect call matters?
                // TODO: Should be intersect_open
                .map(|(v1, v2)| (Segment::new(verts[*v1], verts[*v2]).intersect(&seg_u), *v1, *v2))
                .filter(|(intersection, _v1, _v2)| intersection.is_some())
                .map(|(intersection, v1, v2)| (intersection.unwrap(), v1, v2))
                .collect();

            println!("Intersections: {:?}", intersections);

            for (intersection, v1, v2) in intersections {
                let idx = verts.len();
                verts.push(intersection);
                segs_inter.push(
                    SegWithIntersection {
                        start: v1,
                        end: v2,
                        inter_idx: idx,
                    }
                );
            }

            println!("Segments with intersections: {:?}", segs_inter);

            assert!(segs_inter.len() <=2);
            if let Some(vertex_u) = vertex_u {
                println!("Vertex {:?} is inside the triangle {:?}. Intersections: {}", verts[vertex_u], self.verts(), segs_inter.len());
                assert_eq!(segs_inter.len(), 1);
                new_segs.push((vertex_u, segs_inter[0].inter_idx));
            } else {
                assert!(segs_inter.len() == 0 || segs_inter.len() == 2,
                    "Intersections found: {}", segs_inter.len());
                if segs_inter.len() == 2 {
                    new_segs.push((segs_inter[0].inter_idx, segs_inter[1].inter_idx));
                }
            }
            inters_idx.append(&mut segs_inter);
        }

        println!("Intersections: {:?}", inters_idx);

        println!("New segments after intersection: {:?}", new_segs);

        let mut colinear_verts: Vec<Vec<usize>> = vec![Vec::new(); 9];

        let mut checked = vec![false; 9];
        for i in 0..inters_idx.len() {
            for j in (i+1)..inters_idx.len() {
                if inters_idx[i].start == inters_idx[j].start && inters_idx[i].end == inters_idx[j].end {
                    checked[i] = true;
                    checked[j] = true;
                    let idx1 = inters_idx[i].start;
                    let idx2 = inters_idx[i].end;
                    let idx_i1 = inters_idx[i].inter_idx;
                    let idx_i2 = inters_idx[j].inter_idx;
                    if (verts[idx_i1] - verts[idx1]).mag() < (verts[idx_i2] - verts[idx1]).mag() {
                        colinear_verts[idx1].push(idx_i2);
                        colinear_verts[idx1].push(idx2);
                        colinear_verts[idx_i1].push(idx2);
                        colinear_verts[idx_i2].push(idx1);
                        colinear_verts[idx2].push(idx1);
                        colinear_verts[idx2].push(idx_i1);
                        new_segs.push((idx1, idx_i1));
                        new_segs.push((idx_i1, idx_i2));
                        new_segs.push((idx_i2, idx2));
                    } else {
                        colinear_verts[idx1].push(idx_i1);
                        colinear_verts[idx1].push(idx2);
                        colinear_verts[idx_i2].push(idx2);
                        colinear_verts[idx_i1].push(idx1);
                        colinear_verts[idx2].push(idx1);
                        colinear_verts[idx2].push(idx_i2);
                        new_segs.push((idx1, idx_i2));
                        new_segs.push((idx_i2, idx_i1));
                        new_segs.push((idx_i1, idx2));
                    }
                }
            }
            if !checked[i] {
                new_segs.push((inters_idx[i].start, inters_idx[i].inter_idx));
                new_segs.push((inters_idx[i].inter_idx, inters_idx[i].end));
            }
        }

        println!("New segments after handling colinearity: {:?}", new_segs);
        println!("Colinear vertices: {:?}", colinear_verts);

        for (idx, vert) in verts.clone().into_iter().enumerate() {
            for (other_idx, other_vert) in verts.clone().into_iter().enumerate() {
                if idx == other_idx {
                    continue;
                }
                if colinear_verts[idx].contains(&other_idx) {
                    continue;
                }
                let new_seg = Segment::new(vert, other_vert);
                let mut clash = false;
                for seg in new_segs.inner().clone() {
                    if new_seg.intersect_open(&Segment::new(verts[seg.0], verts[seg.1])).is_some() {
                        clash = true;
                        break;
                    }
                }
                if !clash {
                    new_segs.push((idx, other_idx));
                }
            }
        }

        println!("New segments after adding non-intersecting segments: {:?}", new_segs);

        // Construct triangles from the segments `new_segs`
        let mut segs_map = vec![Vec::new(); 9];
        for seg in new_segs.inner() {
            if !segs_map[seg.0].contains(&seg.1) {
                segs_map[seg.0].push(seg.1);
            }
            if !segs_map[seg.1].contains(&seg.0) {
                segs_map[seg.1].push(seg.0);
            }
        }

        println!("Segment map: {:?}", segs_map);

        let mut tris = TriList::new();

        for i in 0..verts.len() {
            for j in segs_map[i].clone() {
                for k in segs_map[j].clone() {
                    if segs_map[i].contains(&k) {
                        tris.push((i, j, k));
                    }
                }
            }
        }

        println!("Triangles (by vertex indices): {:?}", tris);

        tris.inner().iter().map(|(i, j, k)| Triangle{ verts: [verts[*i], verts[*j], verts[*k]], plane_norm: self.plane_norm}).collect()
    }

}

impl Collide<Point3> for Triangle {
    fn overlap(&self, point: &Point3) -> bool {
        self.vertex_in(*point)
    }
}

impl Collide<Triangle> for Triangle {
    fn overlap(&self, other: &Triangle) -> bool {
        const EPS_INTERSECTION: f64 = 1e-9;

        // 1. Check that triangle planes are parallel
        let planes_allignment = self.plane_norm.dot(&other.plane_norm);
        if planes_allignment.abs() < 0.999 {
            if planes_allignment > 0.0 {
                warn!("Triangles normals face the same way");
            }
            return false;
        }

        // 2. Check that the Aabb of the triangles has an intersection
        if !self.aabb().overlap(&other.aabb()) {
            return false;
        }

        // Choose maximal cross triangle distance
        let cross_triangle_edge = self.verts.
            iter()
            .zip(other.verts.iter())
            .map(|(u,v)| u - v)
            .max_by(|a, b|
                a.dot(a)
                 .partial_cmp(&b.dot(b))
                 .unwrap())
            .unwrap();
        //let cross_triangle_edge = self.verts[ALPHA] - other.verts[ALPHA];

        // 3. Check that the triangles are coplanar
        if self.plane_norm.dot_vec(&cross_triangle_edge) > EPS_INTERSECTION {
            false
        } else {
            // 4. Check that either at least one vertex is inside the triangle or a cross edge
            //    intersection
            for &v in &self.verts {
                if other.vertex_in(v) {
                    return true;
                }
            }

            let segs_u = [
                Segment::new(self.verts[ALPHA], self.verts[BETA]),
                Segment::new(self.verts[BETA], self.verts[GAMMA]),
                Segment::new(self.verts[GAMMA], self.verts[ALPHA]),
            ];
            for seg_u in segs_u {
                if self.overlap(&seg_u) {
                    return true;
                }
            }
            false
        }
    }
}

impl Collide<Segment> for Triangle {
    fn overlap(&self, seg: &Segment) -> bool {
        let segs_u = [
            Segment::new(self.verts[ALPHA], self.verts[BETA]),
            Segment::new(self.verts[BETA], self.verts[GAMMA]),
            Segment::new(self.verts[GAMMA], self.verts[ALPHA]),
        ];
        for seg_u in segs_u {
            // FIXME: Should be open intersection, however this seems to not detect any
            // intersection
            if let Some(_intersection) = seg.intersect(&seg_u) {
                return true;
            }
        }
        false
    }
}

impl Collide<Cube> for Triangle {
    fn overlap(&self, cube: &Cube) -> bool {
        let c = cube.centre();
        let e = cube.half_widths();

        let v0 = self.verts[ALPHA] - c;
        let v1 = self.verts[BETA] - c;
        let v2 = self.verts[GAMMA] - c;

        let f0 = v1 - v0;
        let f1 = v2 - v1;
        let f2 = v0 - v2;

        let u0 = Vec3::x_axis();
        let u1 = Vec3::y_axis();
        let u2 = Vec3::z_axis();

        let axis_test = |axis: &Vec3| {
            let p0 = v0.dot(axis);
            let p1 = v1.dot(axis);
            let p2 = v2.dot(axis);

            let r = e.z().mul_add(
                u2.dot_vec(axis).abs(),
                e.x()
                    .mul_add(u0.dot_vec(axis).abs(), e.y() * u1.dot_vec(axis).abs()),
            );

            if (-(p0.max(p1).max(p2))).max(p0.min(p1).min(p2)) > r {
                return false;
            }

            true
        };

        if !axis_test(&u0.into()) {
            return false;
        }
        if !axis_test(&u1.into()) {
            return false;
        }
        if !axis_test(&u2.into()) {
            return false;
        }

        let axis_u0_f0 = u0.cross_vec(&f0);
        let axis_u0_f1 = u0.cross_vec(&f1);
        let axis_u0_f2 = u0.cross_vec(&f2);

        let axis_u1_f0 = u1.cross_vec(&f0);
        let axis_u1_f1 = u1.cross_vec(&f1);
        let axis_u1_f2 = u1.cross_vec(&f2);

        let axis_u2_f0 = u2.cross_vec(&f0);
        let axis_u2_f1 = u2.cross_vec(&f1);
        let axis_u2_f2 = u2.cross_vec(&f2);

        if !axis_test(&axis_u0_f0) {
            return false;
        }
        if !axis_test(&axis_u0_f1) {
            return false;
        }
        if !axis_test(&axis_u0_f2) {
            return false;
        }

        if !axis_test(&axis_u1_f0) {
            return false;
        }
        if !axis_test(&axis_u1_f1) {
            return false;
        }
        if !axis_test(&axis_u1_f2) {
            return false;
        }

        if !axis_test(&axis_u2_f0) {
            return false;
        }
        if !axis_test(&axis_u2_f1) {
            return false;
        }
        if !axis_test(&axis_u2_f2) {
            return false;
        }

        if !axis_test(&self.plane_norm.into()) {
            return false;
        }

        true
    }
}

impl Trace for Triangle {
    #[inline]
    fn hit(&self, ray: &Ray) -> bool {
        self.intersection_coors(ray).is_some()
    }

    #[inline]
    fn dist(&self, ray: &Ray) -> Option<f64> {
        if let Some((dist, _coors)) = self.intersection_coors(ray) {
            return Some(dist);
        }

        None
    }

    #[inline]
    fn dist_side(&self, ray: &Ray) -> Option<(f64, Side)> {
        self.dist(ray).map(|dist| {
            let side = Side::new(ray.dir(), self.plane_norm);
            (dist, side)
        })
    }
}

impl Transformable for Triangle {
    #[inline]
    fn transform(&mut self, trans: &Trans3) {
        for v in &mut self.verts {
            *v = trans.transform_point(&v.data()).into();
        }

        self.plane_norm = Dir3::from(trans.transform_vector(&self.plane_norm.data()));
    }
}

impl Emit for Triangle {
    fn cast<R: Rng>(&self, rng: &mut R) -> Ray {
        let mut u = rng.random::<f64>();
        let mut v = rng.random::<f64>();

        if (u + v) > 1.0 {
            u = 1.0 - u;
            v = 1.0 - v;
        }

        let edge_a_b = self.verts[BETA] - self.verts[ALPHA];
        let edge_a_c = self.verts[GAMMA] - self.verts[ALPHA];

        let pos = self.verts[ALPHA] + (edge_a_b * u) + (edge_a_c * v);

        Ray::new(pos, self.plane_norm)
    }
}

#[cfg(test)]
mod tests {
    // We implement the transformable for the triangle primitive, so we shall use this for tests.
    use super::{Trans3, Transformable};
    use crate::{geom::{Collide, Trace, Triangle, segment::Segment}, math::Point3};
    use nalgebra::Vector3;
    use std::f64;
    use assert_approx_eq::assert_approx_eq;

    fn unit_triangle() -> Triangle {
        Triangle::new([
            Point3::new(0., 0., 0.),
            Point3::new(1., 0., 0.),
            Point3::new(1., 1., 0.),
        ])
    }

    #[test]
    fn scale_test() {
        let mut tri = unit_triangle();

        // First Scale up.
        tri.transform(&Trans3::new(
            Vector3::new(0., 0., 0.),
            Vector3::new(0., 0., 0.),
            2.0,
        ));
        assert_eq!(
            *tri.verts(),
            [
                Point3::new(0., 0., 0.),
                Point3::new(2., 0., 0.),
                Point3::new(2., 2., 0.),
            ]
        );

        // Now check it is reversible and scale back down.
        tri.transform(&Trans3::new(
            Vector3::new(0., 0., 0.),
            Vector3::new(0., 0., 0.),
            0.25,
        ));
        assert_eq!(
            *tri.verts(),
            [
                Point3::new(0., 0., 0.),
                Point3::new(0.5, 0., 0.),
                Point3::new(0.5, 0.5, 0.),
            ]
        );
    }

    #[test]
    fn translate_test() {
        let mut tri = unit_triangle();

        // First Scale up.
        tri.transform(&Trans3::new(
            Vector3::new(1.5, 1.5, 1.5),
            Vector3::new(0., 0., 0.),
            1.0,
        ));
        assert_eq!(
            *tri.verts(),
            [
                Point3::new(1.5, 1.5, 1.5),
                Point3::new(2.5, 1.5, 1.5),
                Point3::new(2.5, 2.5, 1.5),
            ]
        );

        // Now check it is reversible and scale back down.
        tri.transform(&Trans3::new(
            Vector3::new(-4., -4., -4.),
            Vector3::new(0., 0., 0.),
            1.0,
        ));
        assert_eq!(
            *tri.verts(),
            [
                Point3::new(-2.5, -2.5, -2.5),
                Point3::new(-1.5, -2.5, -2.5),
                Point3::new(-1.5, -1.5, -2.5),
            ]
        );
    }

    #[test]
    fn rotation_test() {
        let mut tri = unit_triangle();

        // Let us rotate around the y Axis by Pi radians (90 degrees).
        tri.transform(&Trans3::new(
            Vector3::new(0., 0., 0.),
            Vector3::y() * f64::consts::FRAC_PI_2,
            1.0,
        ));
        // Check that the components have correctly transformed into the correct axis.
        assert_eq!(tri.verts()[1][2], -1.0);
        assert_eq!(tri.verts()[2][2], -1.0);
    }

    #[test]
    fn perimeter_test() {
        let tri = unit_triangle();
        // Two sides of length 1 and the hypotenuse of length sqrt(2).
        assert_eq!(tri.perimeter(), 2.0 + f64::sqrt(2.0));
    }

    #[test]
    fn centre_test() {
        let tri = unit_triangle();
        assert_eq!(tri.centre(), Point3::new(2. / 3., 1. / 3., 0.));
    }

    #[test]
    fn area_test() {
        let tri = unit_triangle();
        assert_approx_eq!(tri.squared_area(), 0.25);
    }

    #[test]
    #[ignore = "This case is currently not handled correctly."]
    // TODO: this function does not work as expected. Check that the intersection coors are correct.
    fn test_intersection_coords() {
        let tri = unit_triangle();
        let ray = crate::geom::Ray::new(
            Point3::new(0.25, 0.25, 1.0),
            crate::math::Dir3::new(0.0, 0.0, -1.0),
        );
        let (dist, coors) = tri.intersection_coors(&ray).unwrap();
        assert_eq!(dist, 1.0);
        assert_eq!(coors, [0.25, 0.25, 0.0]);
    }

    #[test]
    fn test_intersection_coords_miss() {
        let tri = unit_triangle();
        let ray = crate::geom::Ray::new(
            Point3::new(0.25, 0.25, 1.0),
            crate::math::Dir3::new(0.0, 0.0, 1.0),
        );
        assert!(tri.intersection_coors(&ray).is_none());
    }

    #[test]
    fn hit_miss_test() {
        let tri = unit_triangle();
        // Ray is parallel to the triangle. Will not hit.
        let ray = crate::geom::Ray::new(
            Point3::new(0.25, 0.25, 1.0),
            crate::math::Dir3::new(0.0, 0.0, 1.0),
        );
        assert!(!tri.hit(&ray));

        // Ray is facing into the triangle. It will hit.
        let ray = crate::geom::Ray::new(
            Point3::new(0.25, 0.25, 1.0),
            crate::math::Dir3::new(0.0, 0.0, -1.0),
        );
        assert!(tri.hit(&ray));
    }

    #[test]
    fn test_vertex_in() {
        let tri = Triangle::new([
            Point3::new(0., 0., 0.),
            Point3::new(1., 0., 0.),
            Point3::new(1., 1., 0.),
        ]);

        assert_eq!(tri.vertex_in(Point3::new(-1e-10, 0.0, 0.0)), false);
        assert_eq!(tri.vertex_in(Point3::new(0.0, 0.0, 0.0)), false);
        assert_eq!(tri.vertex_in(Point3::new(1.0, 0.0, 0.0)), false);
        assert_eq!(tri.vertex_in(Point3::new(1.0, 1.0, 0.0)), false);
        assert_eq!(tri.vertex_in(Point3::new(0.5, 0.0, 0.0)), false);
        assert_eq!(tri.vertex_in(Point3::new(0.5, 0.5, 0.0)), false);
        assert_eq!(tri.vertex_in(Point3::new(0.0, 0.0, 1e-10)), false);
        assert_eq!(tri.vertex_in(Point3::new(0.5, 0.3, 0.0)), true);
    }

    #[test]
    fn test_overlaping_triangles() {
        let tri1 = Triangle::new([
            Point3::new(0., 0., 0.),
            Point3::new(1., 0., 0.),
            Point3::new(1., 1., 0.),
        ]);
        let tri2 = Triangle::new([
            Point3::new(0., 1., 0.),
            Point3::new(1., 0., 0.),
            Point3::new(1., 1., 0.),
        ]);

        assert!(tri1.overlap(&tri2));
    }

    #[test]
    fn test_segment_intersect() {
        let seg1 = Segment::new(Point3::new(0., 0., 0.), Point3::new(1., 1., 0.));
        let seg2 = Segment::new(Point3::new(0., 1., 0.), Point3::new(1., 0., 0.));
        let intersection = seg1.intersect(&seg2);
        assert!(intersection.is_some());
        assert_approx_eq!(intersection.unwrap(), Point3::new(0.5, 0.5, 0.));

        let seg1 = Segment::new(Point3::new(1.0, 0.0, 0.0), Point3::new(1.0, 1.0, 0.0));
        let seg2 = Segment::new(Point3::new(0.8, 0.5, 0.0), Point3::new(2.0, 0.0, 0.0));
        assert!(seg1.intersect(&seg2).is_some());
    }

    #[test]
    fn test_segment_intersect_open() {
        let seg1 = Segment::new(Point3::new(0., 0., 0.), Point3::new(1., 1., 0.));
        let seg2 = Segment::new(Point3::new(0., 1., 0.), Point3::new(1., 0., 0.));
        let intersection = seg1.intersect_open(&seg2);
        assert!(intersection.is_some());
        assert_approx_eq!(intersection.unwrap(), Point3::new(0.5, 0.5, 0.));
    }

    #[test]
    fn test_segment_non_intersect_open() {
        let seg1 = Segment::new(Point3::new(0., 0., 0.), Point3::new(1., 1., 0.));
        let seg2 = Segment::new(Point3::new(1., 1., 0.), Point3::new(2., 0., 0.));
        let intersection = seg1.intersect_open(&seg2);
        assert!(intersection.is_none());
    }

    #[test]
    fn test_segment_non_intersect() {
        let seg1 = Segment::new(Point3::new(0., 0., 0.), Point3::new(1., 1., 0.));
        let seg2 = Segment::new(Point3::new(0., 1., 0.), Point3::new(1., 2., 0.));
        let intersection = seg1.intersect(&seg2);
        assert!(intersection.is_none());
    }

    #[test]
    fn test_overlaping_triangles_one_vertex() {
        let tri1 = Triangle::new([
            Point3::new(0., 0., 0.),
            Point3::new(1., 0., 0.),
            Point3::new(1., 1., 0.),
        ]);
        let tri2 = Triangle::new([
            Point3::new(0.8, 0.5, 0.),
            Point3::new(2., 0., 0.),
            Point3::new(2., 1., 0.),
        ]);

        assert!(tri1.overlap(&tri2));

        let new_tri1 = tri1.triangle_split(&tri2);

        println!("New Triangles: {}", new_tri1.len());
        assert_eq!(new_tri1.len(), 5);

    }
}
