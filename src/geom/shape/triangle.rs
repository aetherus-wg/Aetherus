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
use std::{collections::BTreeMap, iter::chain, usize};

use crate::{
    access,
    geom::{segment::Segment, Collide, Cube, Emit, Ray, Side, Split, Trace, Transformable},
    math::{Dir3, Point3, Trans3, Vec3},
    ord::{ALPHA, BETA, GAMMA},
};
use log::{trace, warn};
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
        Self::new_with_norm(verts, plane_norm)
    }

    #[must_use]
    pub fn new_with_norm(verts: [Point3; 3], plane_norm: Dir3) -> Self {
        {
            let a = verts[0] - verts[1];
            let b = verts[1] - verts[2];
            assert!(
                a.cross(&b).abs() > f64::EPSILON,
                "Edges of triangle can't be colinear: {:?}",
                verts
            );
        }
        Self { verts, plane_norm }
    }

    /// Construct triangles from the segments
    pub fn from_verts_segs(&self, segs: &[(usize, usize)], verts: &[Point3]) -> Vec<Self> {
        let mut segs_map = vec![Vec::new(); 9];
        for seg in segs {
            if !segs_map[seg.0].contains(&seg.1) {
                segs_map[seg.0].push(seg.1);
            }
            if !segs_map[seg.1].contains(&seg.0) {
                segs_map[seg.1].push(seg.0);
            }
        }

        trace!("Segment map: {:?}", segs_map);

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

        // Filter out triangles that contain other vertices
        let tris_vec: Vec<_> = tris
            .inner()
            .iter()
            .filter(|(i, j, k)| {
                let tri =
                    Triangle::new_with_norm([verts[*i], verts[*j], verts[*k]], self.plane_norm);
                let mut checked = true;
                for (v_idx, v) in verts.iter().enumerate() {
                    if v_idx != *i && v_idx != *j && v_idx != *k && tri.vertex_in(v.clone()) {
                        checked = false;
                        break;
                    }
                }
                checked
            })
            .collect();

        trace!("Triangles (by vertex indices): {:?}", tris_vec);

        tris_vec
            .iter()
            .map(|(i, j, k)| {
                Triangle::new_with_norm([verts[*i], verts[*j], verts[*k]], self.plane_norm)
            })
            .collect()
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
        let mins = self.verts.iter().fold(
            Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY),
            |acc, v| Point3::new(acc.x().min(v.x()), acc.y().min(v.y()), acc.z().min(v.z())),
        );
        let maxs = self.verts.iter().fold(
            Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
            |acc, v| Point3::new(acc.x().max(v.x()), acc.y().max(v.y()), acc.z().max(v.z())),
        );
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

    pub fn edges(&self) -> [Segment; 3] {
        [
            Segment::new(self.verts[BETA], self.verts[GAMMA]),
            Segment::new(self.verts[GAMMA], self.verts[ALPHA]),
            Segment::new(self.verts[ALPHA], self.verts[BETA]),
        ]
    }

    pub fn vertex_in(&self, vertex: Point3) -> bool {
        let segs = self.edges();
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
                trace!("Vertex {:?} is on edge {:?} of triangle.", vertex, seg);
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

impl Collide<Point3> for Triangle {
    // NOTE: Similar to Triangle.vertex_in, but including coincidence with triangle vertices and
    // edges
    fn overlap(&self, point: &Point3) -> bool {
        let segs = self.edges();
        let mut sign: Option<bool> = None;

        // 1. First check that vertex is in the plane of the triangle
        let vertex_to_tri = point - self.verts[ALPHA];
        if self.plane_norm.dot_vec(&vertex_to_tri).abs() > 1e-9 {
            return false;
        }

        // 2. Then check that vertex is on the same side of all edges of the triangle
        for seg in &segs {
            let edge_vec = seg.end - seg.start;
            let to_vertex_vec = point - seg.start;
            if to_vertex_vec.norm1() < 1e-9 {
                // Vertex coincides with triangle vertex
                break;
            }
            let cross_prod = edge_vec.cross(&to_vertex_vec);
            // 3. Check that the vertex is on the same side of each segment
            let cross_prod_mag = self.plane_norm.dot_vec(&cross_prod);
            if cross_prod_mag.abs() < 1e-9 {
                // Point is on the edge
                continue;
            }
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
}

impl Collide<Triangle> for Triangle {
    fn overlap(&self, other: &Triangle) -> bool {
        const EPS_INTERSECTION: f64 = 1e-9;

        // 1. Check that triangle planes are parallel
        let planes_allignment = self.plane_norm.dot(&other.plane_norm);
        if planes_allignment.abs() < 0.999 {
            return false;
        }

        // 2. Check that the Aabb of the triangles has an intersection
        if !self.aabb().overlap(&other.aabb()) {
            return false;
        }

        // Choose maximal cross triangle distance
        let cross_triangle_edge = self
            .verts
            .iter()
            .zip(other.verts.iter())
            .map(|(u, v)| u - v)
            .max_by(|a, b| a.dot(a).partial_cmp(&b.dot(b)).unwrap())
            .unwrap();
        //let cross_triangle_edge = self.verts[ALPHA] - other.verts[ALPHA];

        // 3. Check that the triangles are coplanar
        if self.plane_norm.dot_vec(&cross_triangle_edge).abs() > EPS_INTERSECTION {
            false
        } else {
            if planes_allignment > 0.0 {
                warn!(
                    "Triangles normals face the same way for: {:?} and {:?}",
                    self, other
                );
            }

            // 4. Check that either at least one vertex is inside the triangle or a cross edge
            //    intersection
            for &v in &other.verts {
                if self.vertex_in(v) {
                    trace!("Found vertex {:?} inside triangle", v);
                    return true;
                }
            }
            let segs_u = [
                Segment::new(other.verts[ALPHA], other.verts[BETA]),
                Segment::new(other.verts[BETA], other.verts[GAMMA]),
                Segment::new(other.verts[GAMMA], other.verts[ALPHA]),
            ];
            for seg_u in segs_u {
                // FIXME: This should exclude touching but not overlaping triangles
                if self.overlap(&seg_u) {
                    trace!("Found edge {:?} intersection with triangle", seg_u);
                    return true;
                }
            }
            false
        }
    }
}

impl Collide<Segment> for Triangle {
    fn overlap(&self, seg: &Segment) -> bool {
        let segs_u = self.edges();
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

#[derive(Debug)]
pub struct SplitSegments {
    pub segs: Vec<(usize, usize)>,
    pub verts: Vec<Point3>,
}

impl Collide<SplitSegments> for Triangle {
    fn overlap(&self, other: &SplitSegments) -> bool {
        other
            .segs
            .iter()
            .map(|(v0, v1)| Segment::new(other.verts[*v0], other.verts[*v1]))
            .any(|seg| self.overlap(&seg))
    }
}

impl Split<SplitSegments, ()> for Triangle {
    type Inst = Vec<Self>;
    fn split_transparent(&self, other: &SplitSegments) -> (Self::Inst, ()) {
        trace!("Splitting triangle with segments: {:?} and verts: {:?}", other.segs, other.verts);
        let segs: Vec<_> = other.segs.iter().map(|(v0, v1)| (v0 + 3, v1 + 3)).collect(); // Offset indices by 3
        let mut verts_remap: [isize; 12] = [-1; 12];
        let mut offset = 0;
        for (i, v) in other.verts.iter().enumerate() {
            let other_idx = i + 3;
            let idx = self
                .verts
                .iter()
                .position(|&vert| (vert - *v).norm1() < 1e-9);
            let valid = self.overlap(v);
            if other_idx < 12 {
                if valid {
                    match idx {
                        Some(idx) => {
                            offset += 1;
                            verts_remap[other_idx] = idx as isize;
                            trace!("Moving original {} to {}", i, idx)
                        }
                        None => {
                            verts_remap[other_idx] = other_idx as isize - offset;
                        }
                    }
                } else {
                    trace!("Discarding vertex {} since it's not overlapping the triangle at {:?}", i, v);
                    offset += 1;
                }
            }
        }

        trace!("Remaping {:?}", verts_remap);

        let mut segs: Vec<_> = segs
            .iter()
            .filter_map(|(v0, v1)| {
                if verts_remap[*v0] >= 0 && verts_remap[*v1] >= 0 {
                    let new_v0 = verts_remap[*v0] as usize;
                    let new_v1 = verts_remap[*v1] as usize;
                    Some((new_v0, new_v1))
                } else {
                    None
                }
            })
            .collect();
        let verts = chain(
            self.verts.iter().map(|v| v.clone()),
            other.verts.iter().enumerate().filter_map(|(i, v)| {
                let other_idx = i + 3;
                if verts_remap[other_idx] >= 3 {
                    Some(v.clone())
                } else {
                    None
                }
            }),
        )
        .collect::<Vec<_>>();

        trace!("Initial segments after remapping: {:?}\nwith Verts: {:?}", segs, verts);

        // Solving colinear vertices and generate segs that don't overlap each other
        let mut colinear_map: BTreeMap<(usize, usize), Vec<usize>> = BTreeMap::new();
        for i in 3..verts.len() {
            [(0, 1), (1, 2), (2, 0)].iter().for_each(|(a, b)| {
                let seg = Segment::new(verts[*a], verts[*b]);
                if seg.overlap(&verts[i]) {
                    trace!("Vertex {:?} is on edge {:?} of triangle.", verts[i], seg);
                    colinear_map
                        .entry((*a, *b))
                        .or_insert_with(Vec::new)
                        .push(i);
                }
            });
        }

        let mut colinear_blacklist: Vec<Vec<usize>> = vec![Vec::new(); 9];
        for ((a, b), colinear_verts_in) in &colinear_map {
            let mut colinear_verts = Vec::new();
            colinear_verts.push(*a);
            colinear_verts.push(*b);
            colinear_verts_in.iter().for_each(|v_idx| {
                colinear_verts.push(*v_idx);
            });
            colinear_verts.sort_by(|v_idx_0, v_idx_1| {
                let edge_seg = Segment::new(verts[*a], verts[*b]);
                let alpha_0 = edge_seg.parametric_dist(&verts[*v_idx_0]);
                let alpha_1 = edge_seg.parametric_dist(&verts[*v_idx_1]);
                alpha_0
                    .partial_cmp(&alpha_1)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }); // Sort by alpha
            for i in 0..colinear_verts.len() {
                let v_i = colinear_verts[i];
                if i > 0 {
                    let v_left = colinear_verts[i - 1];
                    if !segs.contains(&(v_left, v_i)) && !segs.contains(&(v_i, v_left)) {
                        segs.push((v_left, v_i));
                    }
                }
                if i < colinear_verts.len() - 1 {
                    let v_right = colinear_verts[i + 1];
                    if !segs.contains(&(v_right, v_i)) && !segs.contains(&(v_i, v_right)) {
                        segs.push((v_i, v_right));
                        trace!(
                            "Adding segment between colinear vertices: {} and {}",
                            v_right,
                            v_i
                        );
                    }
                }
                for j in 0..colinear_verts.len() {
                    if ((j as isize) < (i as isize - 1)) || (j > i + 1) {
                        let v_j = colinear_verts[j];
                        if !colinear_blacklist[v_j].contains(&v_i) {
                            colinear_blacklist[v_j].push(v_i);
                        }
                        if !colinear_blacklist[v_i].contains(&v_j) {
                            colinear_blacklist[v_i].push(v_j);
                        }
                    }
                }
            }
        }


        trace!("Colinear blacklist: {:?}", colinear_blacklist);

        for (idx, vert) in verts.clone().into_iter().enumerate() {
            // NOTE: Assume all verts are valid since we've already pruned the one that are not
            // overlaping the triangle
            for (other_idx, other_vert) in verts.clone().into_iter().enumerate() {
                if idx == other_idx {
                    continue;
                }
                if idx > other_idx {
                    // Only consider the pair once
                    continue;
                }
                if colinear_blacklist[idx].contains(&other_idx) {
                    continue;
                }
                if segs.contains(&(idx, other_idx)) || segs.contains(&(other_idx, idx)) {
                    continue;
                }
                let new_seg = Segment::new(vert, other_vert);
                let mut clash = false;
                for seg in segs.iter() {
                    match Segment::new(verts[seg.0], verts[seg.1])
                        .intersect_with_eps(&new_seg, 1e-9)
                    {
                        Some((_intersect, false)) => {
                            clash = true;
                            break;
                        }
                        _ => {}
                    }
                }
                if !clash {
                    segs.push((idx, other_idx));
                }
            }
        }

        trace!(
            "New segments after adding non-intersecting segments: {:?}",
            segs
        );

        let tris_vec = self.from_verts_segs(&segs, &verts);

        assert!(
            tris_vec.len() <= 7,
            "Expected at most 7 triangles, but {} where formed",
            tris_vec.len()
        );

        (tris_vec, ())
    }

    #[inline]
    fn split(&self, other: &SplitSegments) -> Self::Inst {
        self.split_transparent(other).0
    }
}

impl Split<Triangle, SplitSegments> for Triangle {
    // TODO: Also need to solve non-coplanar triangles that intersect forming one and only
    // intersecting segment
    type Inst = Vec<Self>;
    fn split_transparent(&self, other: &Triangle) -> (Self::Inst, SplitSegments) {
        trace!("Check intersections of {:?} with {:?}", self, other);

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
            fn inner(&self) -> &[(usize, usize)] {
                &self.segs
            }
        }

        let mut verts = vec![self.verts[ALPHA], self.verts[BETA], self.verts[GAMMA]];
        let mut verts_valid = vec![true, true, true];

        let segs_u: [(usize, usize); 3] = [(0, 1), (1, 2), (2, 0)];

        let mut new_segs = SegList::new();

        let mut verts_in = vec![];
        let mut verts_out = vec![];
        let mut segs_v = vec![];

        // Resolve vertices and segments inside the triangle
        for &v in &other.verts {
            if self.vertex_in(v) {
                verts_in.push(verts.len());
                verts.push(v);
                verts_valid.push(true);
            } else {
                verts_out.push(v);
            }
        }
        for i in 0..verts_in.len() {
            for j in (i + 1)..verts_in.len() {
                new_segs.push((verts_in[i], verts_in[j]));
            }
        }
        trace!("Verts in: {:?}, Verts out: {:?}", verts_in, verts_out);
        match verts_out.len() {
            0 => {
                assert_eq!(verts_in.len(), 3);
                // No vertices outside. The other triangle is fully contained in this one.
            }
            1 => {
                assert_eq!(verts_in.len(), 2);
                segs_v.push((
                    Some(verts_in[0]),
                    Segment::new(verts_out[0], verts[verts_in[0]]),
                ));
                segs_v.push((
                    Some(verts_in[1]),
                    Segment::new(verts_out[0], verts[verts_in[1]]),
                ));
            }
            2 => {
                assert_eq!(verts_in.len(), 1);
                segs_v.push((
                    Some(verts_in[0]),
                    Segment::new(verts_out[0], verts[verts_in[0]]),
                ));
                segs_v.push((
                    Some(verts_in[0]),
                    Segment::new(verts_out[1], verts[verts_in[0]]),
                ));
            }
            3 => {
                assert_eq!(verts_in.len(), 0);
                segs_v.push((None, Segment::new(verts_out[0], verts_out[1])));
                segs_v.push((None, Segment::new(verts_out[1], verts_out[2])));
                segs_v.push((None, Segment::new(verts_out[2], verts_out[0])));
            }
            _ => unreachable!(),
        }

        trace!("Segments in triangle: {:?}", segs_v);

        // Find all intersections and segments that need to be included in the new triangles
        #[derive(Debug, Clone, Copy)]
        struct SegWithIntersection {
            start: usize,
            end: usize,
            inter_idx: usize,
            eps: bool,
        }

        impl SegWithIntersection {
            fn is_bound(&self) -> bool {
                self.start == self.inter_idx || self.end == self.inter_idx
            }
        }

        let mut inters_idx: Vec<SegWithIntersection> = Vec::new();

        for (vertex_v, seg_v) in segs_v {
            trace!(
                "Vertex {:?} is inside the triangle. Segment: {:?}",
                vertex_v,
                seg_v
            );
            let mut segs_all_inter = Vec::new();
            let intersections: Vec<_> = segs_u
                .iter()
                // TODO: Check if order of interesect call matters?
                // TODO: Should be intersect_open
                .map(|(u1, u2)| {
                    (
                        Segment::new(verts[*u1], verts[*u2]).intersect_with_eps(&seg_v, 1e-9),
                        *u1,
                        *u2,
                    )
                })
                .filter(|(intersection, _u1, _u2)| intersection.is_some())
                .map(|(intersection, u1, u2)| (intersection.unwrap(), u1, u2))
                .collect();

            trace!("Intersections: {:?}", intersections);

            let mut inter_indices = Vec::new();
            for (intersection, u1, u2) in intersections.clone() {
                let idx = if (verts[u1] - intersection.0).norm1() < 1e-9 {
                    u1
                } else if (verts[u2] - intersection.0).norm1() < 1e-9 {
                    u2
                } else {
                    let mut idx = verts.len();
                    for (i, v) in verts.iter().enumerate() {
                        if (v - intersection.0).norm1() < 1e-9 {
                            idx = i;
                            break;
                        }
                    }
                    idx
                };

                if !inter_indices.contains(&idx) {
                    // Discard segments that are coming from outside the triangle and are fuzzy
                    // intersections
                    inter_indices.push(idx);
                    if idx != u1 && idx != u2 {
                        verts.push(intersection.0);
                        verts_valid.push(false);
                    }
                    segs_all_inter.push(SegWithIntersection {
                        start: u1,
                        end: u2,
                        inter_idx: idx,
                        eps: intersection.1,
                    });
                }
            }

            assert!(
                segs_all_inter.len() <= 2,
                "Expected at most 2 intersections, but {} where found.",
                segs_all_inter.len()
            );

            let segs_inter: Vec<_> = segs_all_inter
                .iter()
                .filter(|seg_inter| {
                    vertex_v.is_some() || !seg_inter.eps || segs_all_inter.len() == 2
                })
                .collect();

            segs_inter.iter().for_each(|seg_inter| {
                verts_valid[seg_inter.inter_idx] = true;
            });

            trace!(
                "Segments with intersections: {:?}, expected max 2",
                segs_inter
            );
            trace!("Vertices {:?} with validity {:?}", verts, verts_valid);

            assert!(
                segs_inter.len() <= 2,
                "Intersections detected {}",
                segs_inter.len()
            );
            if let Some(vertex_v) = vertex_v {
                trace!(
                    "Vertex {}:{:?} is inside the triangle {:?}. Intersections: {}",
                    vertex_v,
                    verts[vertex_v],
                    self.verts(),
                    segs_inter.len()
                );
                assert_eq!(segs_inter.len(), 1);
                new_segs.push((vertex_v, segs_inter[0].inter_idx));
            } else {
                if segs_inter.len() == 2 {
                    new_segs.push((segs_inter[0].inter_idx, segs_inter[1].inter_idx));
                } else {
                    // FIXME:
                    if segs_inter.len() == 1 {
                        trace!("WARN: Outer intersection with eps error must be checked");
                    }
                }
            }
            inters_idx.extend(segs_inter);
        }

        // TODO: Deduplicate inter_idx among segs_inter
        let mut inter_indices = Vec::new();
        let mut i = 0;
        while i < inters_idx.len() {
            if inter_indices.contains(&inters_idx[i].inter_idx) {
                inters_idx.remove(i);
            } else {
                inter_indices.push(inters_idx[i].inter_idx);
                i += 1;
            }
        }
        trace!("Intersections: {:?}", inters_idx);

        trace!("New segments after intersection: {:?}", new_segs);

        let mut colinear_verts: Vec<Vec<usize>> = vec![Vec::new(); 9];

        let mut checked = vec![false; 9];
        for i in 0..inters_idx.len() {
            trace!("Find colinear verts for vertex {}", inters_idx[i].inter_idx);
            // If the intersection is one of triangle vertices, discard new segments being
            // created
            if inters_idx[i].is_bound() {
                continue;
            }
            for j in (i + 1)..inters_idx.len() {
                // TODO: Check how intersection match with triangles vertices affects this branch
                if inters_idx[i].start == inters_idx[j].start
                    && inters_idx[i].end == inters_idx[j].end
                    && !inters_idx[j].is_bound()
                {
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
                colinear_verts[inters_idx[i].start].push(inters_idx[i].end);
                colinear_verts[inters_idx[i].end].push(inters_idx[i].start);
                new_segs.push((inters_idx[i].start, inters_idx[i].inter_idx));
                new_segs.push((inters_idx[i].inter_idx, inters_idx[i].end));
            }
        }

        trace!("New segments after handling colinearity: {:?}", new_segs);
        trace!("Colinear vertices: {:?}", colinear_verts);

        for (idx, vert) in verts.clone().into_iter().enumerate() {
            if !verts_valid[idx] {
                continue;
            }
            for (other_idx, other_vert) in verts.clone().into_iter().enumerate() {
                if !verts_valid[other_idx] {
                    continue;
                }
                if idx == other_idx {
                    continue;
                }
                if colinear_verts[idx].contains(&other_idx) {
                    continue;
                }
                let new_seg = Segment::new(vert, other_vert);
                let mut clash = false;
                for seg in new_segs.inner() {
                    match Segment::new(verts[seg.0], verts[seg.1])
                        .intersect_with_eps(&new_seg, 1e-9)
                    {
                        Some((_intersect, false)) => {
                            clash = true;
                            break;
                        }
                        _ => {}
                    }
                }
                if !clash {
                    new_segs.push((idx, other_idx));
                }
            }
        }

        trace!(
            "New segments after adding non-intersecting segments: {:?}",
            new_segs
        );

        let tris_vec = self.from_verts_segs(new_segs.inner(), &verts);

        assert!(
            tris_vec.len() <= 7,
            "Expected at most 7 triangles, but {} where formed",
            tris_vec.len()
        );

        let split_segments = SplitSegments {
            segs: new_segs.inner().to_vec(),
            verts,
        };

        (tris_vec, split_segments)
    }

    #[inline]
    fn split(&self, other: &Triangle) -> Self::Inst {
        self.split_transparent(other).0
    }
}

#[cfg(test)]
mod tests {
    // We implement the transformable for the triangle primitive, so we shall use this for tests.
    use super::{Trans3, Transformable};
    use crate::{
        geom::{segment::Segment, Collide, Split, Trace, Triangle},
        math::Point3,
    };
    use assert_approx_eq::assert_approx_eq;
    use nalgebra::Vector3;
    use std::f64;

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
    fn test_non_overlaping_triangles() {
        let tri1 = Triangle::new([
            Point3::new(0., 0., 0.),
            Point3::new(1., 0., 0.),
            Point3::new(1., 1., 0.),
        ]);

        // Not coplanar
        let tri2 = Triangle::new([
            Point3::new(0., 1., 1.),
            Point3::new(1., 0., 1.),
            Point3::new(1., 1., 1.),
        ]);
        assert!(!tri1.overlap(&tri2));

        // Coplanar but not overlapping
        let tri2 = Triangle::new([
            Point3::new(0., 0.1, 0.),
            Point3::new(0., 1., 0.),
            Point3::new(0.9, 1., 0.),
        ]);
        assert!(!tri1.overlap(&tri2));

        // Intersecting, but not overlapping in the same plane
        let tri2 = Triangle::new([
            Point3::new(0., 0., -1.),
            Point3::new(1., 0., 1.),
            Point3::new(1., 1., 1.),
        ]);
        assert!(!tri1.overlap(&tri2));
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

        let new_tri1 = tri1.split(&tri2);

        println!("New Triangles: {}", new_tri1.len());
        assert_eq!(new_tri1.len(), 5);
    }

    // Test all triangles split choices
    // 1. resolved => (1,1)
    // 2. edge colinear:
    //   a) 2 edges colinear => (3, 1)
    //   b) vertex out => (1,1)
    //   c) vertex in:
    //     i) v1=u1 and v2=u2 => (3,1)
    //     ii) v1 = u1, v2 in (u1, u2) => (4,1)
    //     iii) {v1,v2} in (u1, u2) => (5,1)
    // 3. vertex in = 1 (v1)
    //   a) [v1,v2] and [v1,v3] intersects same U edge => (5,3)
    //   b) [v1,v2] and [v1, v2] intersects distinct U edges => (5, 5)
    //   c) One edge of V tri contains U vertex => (4, 3)
    //   d) Two edges of V tri contains U vertices => (3, 3)
    // 4. vertex in = 2 (v1,v2)
    //   a) [v1,v3] and [v2,v3] intersects U edges
    //     i) same U edge => (7,3)
    //     ii) 2 distinct U edges => (7, 5)
    //   b) One edge of V contains U vertex => (6, 3)
    // 5. vertex in = 3 => (7,1)
    // 6. vertex in = 0
    //   a) 2 edges of V each intersect with any 2 edges of U
    //     i) Vertex of U inside V => (5, 7)
    //     ii) Vertex of U on edge of V => (5, 6)
    //   b) 1 edge of V intersects with 2 edges of U
    //     i) One vertex of U on edge of V => (3, 6)
    //     ii) Two vertices of U on edges of V => (3, 5)
    //     iii) Two vertices of U inside V => (3, 7)
    //   c) 3 edges of V intersect 3 edges of U => (7,7)

    fn count_colinear(us: &[Segment; 3], vs: &[Segment; 3]) -> usize {
        let mut cnt = 0;
        for u in us {
            for v in vs {
                if u.colinear(&v) {
                    cnt += 1;
                }
            }
        }
        assert!(
            cnt <= 3,
            "Can't have more than 3 colinear cross edges between 2 triangles, but counted {}",
            cnt
        );
        cnt
    }

    fn count_verts_in(u_tri: &Triangle, v_tri: &Triangle) -> usize {
        v_tri
            .verts()
            .iter()
            .fold(0, |cnt, v| if u_tri.vertex_in(*v) { cnt + 1 } else { cnt })
    }

    #[test]
    fn test_matching_triangles_split() {
        let tri1 = Triangle::new([
            Point3::new(0., 0., 0.),
            Point3::new(1., 0., 0.),
            Point3::new(1., 1., 0.),
        ]);
        let tri2 = tri1.clone();

        assert!(tri1.overlap(&tri2));
        assert_eq!(count_colinear(&tri1.edges(), &tri2.edges()), 3);

        let (new_tri1, split_segs) = tri1.split_transparent(&tri2);
        assert_eq!(new_tri1.len(), 1);
        assert_eq!(new_tri1[0], tri1);

        let new_tri2 = tri2.split(&split_segs);
        assert_eq!(new_tri2.len(), 1);
        assert_eq!(new_tri2[0], tri2);
    }

    #[test]
    fn test_colinear_edges() {
        let tri_u = Triangle::new([
            Point3::new(0., 0., 1.),
            Point3::new(0., 0., -1.),
            Point3::new(0., 1., 0.),
        ]);
        let segs_u = tri_u.edges();

        // a) i)
        let tri_v = Triangle::new([
            Point3::new(0., 0., 1.),
            Point3::new(0., 0., 0.),
            Point3::new(0., 0.5, 0.5),
        ]);
        let segs_v = tri_v.edges();
        assert_eq!(count_colinear(&segs_u, &segs_v), 2);
        let (new_tris_u, split_segs) = tri_u.split_transparent(&tri_v);
        assert_eq!(new_tris_u.len(), 3);
        assert_eq!(tri_v.split(&split_segs).len(), 1);

        // a) ii)
        let tri_v = Triangle::new([
            Point3::new(0., 0., 1.),
            Point3::new(0., 0., -1.),
            Point3::new(0., 0.5, 0.5),
        ]);
        let segs_v = tri_v.edges();
        assert_eq!(count_colinear(&segs_u, &segs_v), 2);
        let (new_tris_u, split_segs) = tri_u.split_transparent(&tri_v);
        assert_eq!(new_tris_u.len(), 2);
        assert_eq!(tri_v.split(&split_segs).len(), 1);

        // b)
        let tri_v = Triangle::new([
            Point3::new(0., 0., 1.),
            Point3::new(0., 0., -0.8),
            Point3::new(0., -1., 0.),
        ]);
        let segs_v = tri_v.edges();
        assert_eq!(count_colinear(&segs_u, &segs_v), 1);
        let (new_tris_u, split_segs) = tri_u.split_transparent(&tri_v);
        assert_eq!(new_tris_u.len(), 1);
        assert_eq!(tri_v.split(&split_segs).len(), 1);

        // c) vert in
        // i) // WARN: Duplicate of vert_in=1 d)
        let tri_v = Triangle::new([
            Point3::new(0., 0., 1.),
            Point3::new(0., 0., -1.),
            Point3::new(0., 0.5, 0.),
        ]);
        let segs_v = tri_v.edges();
        assert_eq!(count_colinear(&segs_u, &segs_v), 1);
        let (new_tris_u, split_segs) = tri_u.split_transparent(&tri_v);
        assert_eq!(new_tris_u.len(), 3);
        assert_eq!(tri_v.split(&split_segs).len(), 1);

        // ii) // WARN: Duplicate of vert_in=1 c)
        let tri_v = Triangle::new([
            Point3::new(0., 0., 1.),
            Point3::new(0., 0., -0.5),
            Point3::new(0., 0.5, 0.),
        ]);
        let segs_v = tri_v.edges();
        assert_eq!(count_colinear(&segs_u, &segs_v), 1);
        let (new_tris_u, split_segs) = tri_u.split_transparent(&tri_v);
        assert_eq!(new_tris_u.len(), 4);
        assert_eq!(tri_v.split(&split_segs).len(), 1);

        // iii) // WARN: Duplicate of vert_in=1 b)
        let tri_v = Triangle::new([
            Point3::new(0., 0., 0.5),
            Point3::new(0., 0., -0.5),
            Point3::new(0., 0.5, 0.),
        ]);
        let segs_v = tri_v.edges();
        assert_eq!(count_colinear(&segs_u, &segs_v), 1);
        let (new_tris_u, split_segs) = tri_u.split_transparent(&tri_v);
        assert_eq!(new_tris_u.len(), 5);
        assert_eq!(tri_v.split(&split_segs).len(), 1);

        // iv) All  vertices of V on edges of U
        let tri_v = Triangle::new([
            Point3::new(0., 0., 0.5),
            Point3::new(0., 0., -0.5),
            Point3::new(0., 0.5, 0.5),
        ]);
        let segs_v = tri_v.edges();
        assert_eq!(count_colinear(&segs_u, &segs_v), 1);
        let (new_tris_u, split_segs) = tri_u.split_transparent(&tri_v);
        assert_eq!(new_tris_u.len(), 4);
        assert_eq!(tri_v.split(&split_segs).len(), 1);
    }

    #[test]
    fn test_one_vert_in() {
        let tri_u = Triangle::new([
            Point3::new(0., 0.5, 0.5),
            Point3::new(0., 0.5, -0.5),
            Point3::new(0., 1.5, 0.),
        ]);

        // a)
        let tri_v = Triangle::new([
            Point3::new(0., 1., 0.),
            Point3::new(0., 0., 0.5),
            Point3::new(0., 0., -0.5),
        ]);
        assert_eq!(count_verts_in(&tri_u, &tri_v), 1);
        let (new_tris_u, split_segs) = tri_u.split_transparent(&tri_v);
        assert_eq!(new_tris_u.len(), 5);
        assert_eq!(tri_v.split(&split_segs).len(), 3);

        // b)
        let tri_v = Triangle::new([
            Point3::new(0., 1., 0.),
            Point3::new(0., 2., 0.5),
            Point3::new(0., 2., -0.5),
        ]);
        assert_eq!(count_verts_in(&tri_u, &tri_v), 1);
        let (new_tris_u, split_segs) = tri_u.split_transparent(&tri_v);
        assert_eq!(new_tris_u.len(), 5);
        assert_eq!(tri_v.split(&split_segs).len(), 5);

        // c)
        let tri_v = Triangle::new([
            Point3::new(0., 1., 0.),
            Point3::new(0., 0., 0.5),
            Point3::new(0., 0., -1.),
        ]);
        assert_eq!(count_verts_in(&tri_u, &tri_v), 1);
        let (new_tris_u, split_segs) = tri_u.split_transparent(&tri_v);
        assert_eq!(new_tris_u.len(), 4);
        assert_eq!(tri_v.split(&split_segs).len(), 3);

        // d)
        let tri_v = Triangle::new([
            Point3::new(0., 1., 0.),
            Point3::new(0., 0., 1.),
            Point3::new(0., 0., -1.),
        ]);
        assert_eq!(count_verts_in(&tri_u, &tri_v), 1);
        let (new_tris_u, split_segs) = tri_u.split_transparent(&tri_v);
        assert_eq!(new_tris_u.len(), 3);
        assert_eq!(tri_v.split(&split_segs).len(), 3);
    }

    #[test]
    fn test_two_verts_in() {
        let tri_u = Triangle::new([
            Point3::new(0., 2., 0.),
            Point3::new(0., 0., -1.),
            Point3::new(0., 0., 1.),
        ]);

        // a) i)
        let tri_v = Triangle::new([
            Point3::new(0., 0.5, 0.5),
            Point3::new(0., 0.5, -0.5),
            Point3::new(0., -1., 0.),
        ]);
        assert_eq!(count_verts_in(&tri_u, &tri_v), 2);
        let (new_tris_u, split_segs) = tri_u.split_transparent(&tri_v);
        assert_eq!(new_tris_u.len(), 7);
        assert_eq!(tri_v.split(&split_segs).len(), 3);

        // a) ii)
        let tri_v = Triangle::new([
            Point3::new(0., 0.5, 0.5),
            Point3::new(0., 0.5, -0.5),
            Point3::new(0., 3., 0.),
        ]);
        assert_eq!(count_verts_in(&tri_u, &tri_v), 2);
        let (new_tris_u, split_segs) = tri_u.split_transparent(&tri_v);
        assert_eq!(new_tris_u.len(), 7);
        assert_eq!(tri_v.split(&split_segs).len(), 5);

        // b)
        let tri_v = Triangle::new([
            Point3::new(0., 0.5, 0.5),
            Point3::new(0., 0.5, 0.),
            Point3::new(0., 3., 0.),
        ]);
        assert_eq!(count_verts_in(&tri_u, &tri_v), 2);
        let (new_tris_u, split_segs) = tri_u.split_transparent(&tri_v);
        assert_eq!(new_tris_u.len(), 6);
        assert_eq!(tri_v.split(&split_segs).len(), 3);
    }

    #[test]
    fn test_verts_out() {
        let tri_u = Triangle::new([
            Point3::new(0., 1., 0.5),
            Point3::new(0., 1., -0.5),
            Point3::new(0., -1., 0.),
        ]);

        // a) i)
        let tri_v = Triangle::new([
            Point3::new(0., 2., 0.),
            Point3::new(0., 0., -0.4),
            Point3::new(0., 0., 1.5),
        ]);
        assert_eq!(count_verts_in(&tri_u, &tri_v), 0);
        let (new_tris_u, split_segs) = tri_u.split_transparent(&tri_v);
        assert_eq!(new_tris_u.len(), 5);
        assert_eq!(tri_v.split(&split_segs).len(), 7);

        // a) ii)
        let tri_v = Triangle::new([
            Point3::new(0., 2., 0.),
            Point3::new(0., 0., -0.4),
            Point3::new(0., 0., 1.),
        ]);
        assert_eq!(count_verts_in(&tri_u, &tri_v), 0);
        let (new_tris_u, split_segs) = tri_u.split_transparent(&tri_v);
        assert_eq!(new_tris_u.len(), 5);
        assert_eq!(tri_v.split(&split_segs).len(), 6);

        // b) i)
        let tri_v = Triangle::new([
            Point3::new(0., 2., 0.),
            Point3::new(0., 0., -1.),
            Point3::new(0., 0., 1.5),
        ]);
        assert_eq!(count_verts_in(&tri_u, &tri_v), 0);
        let (new_tris_u, split_segs) = tri_u.split_transparent(&tri_v);
        assert_eq!(new_tris_u.len(), 3);
        assert_eq!(tri_v.split(&split_segs).len(), 6);

        // b) ii)
        let tri_v = Triangle::new([
            Point3::new(0., 2., 0.),
            Point3::new(0., 0., -1.),
            Point3::new(0., 0., 1.),
        ]);
        assert_eq!(count_verts_in(&tri_u, &tri_v), 0);
        let (new_tris_u, split_segs) = tri_u.split_transparent(&tri_v);
        assert_eq!(new_tris_u.len(), 3);
        assert_eq!(tri_v.split(&split_segs).len(), 5);

        // b) iii)
        let tri_v = Triangle::new([
            Point3::new(0., 2., 0.),
            Point3::new(0., 0., -1.5),
            Point3::new(0., 0., 1.5),
        ]);
        assert_eq!(count_verts_in(&tri_u, &tri_v), 0);
        let (new_tris_u, split_segs) = tri_u.split_transparent(&tri_v);
        assert_eq!(new_tris_u.len(), 3);
        assert_eq!(tri_v.split(&split_segs).len(), 7);

        // c)
        let tri_v = Triangle::new([
            Point3::new(0., 1.1, 0.),
            Point3::new(0., 0., -1.),
            Point3::new(0., 0., 1.),
        ]);
        assert_eq!(count_verts_in(&tri_u, &tri_v), 0);
        let (new_tris_u, split_segs) = tri_u.split_transparent(&tri_v);
        assert_eq!(new_tris_u.len(), 7);
        assert_eq!(tri_v.split(&split_segs).len(), 7);
    }

    #[test]
    fn test_segments_split() {
        let tri_u = Triangle::new([
            Point3::new(0., 2., 0.),
            Point3::new(0., 0., -1.),
            Point3::new(0., 0., 1.),
        ]);

        // a) ii)
        let tri_v = Triangle::new([
            Point3::new(0., 0.5, 0.5),
            Point3::new(0., 0.5, -0.5),
            Point3::new(0., 3., 0.),
        ]);
        assert_eq!(count_verts_in(&tri_u, &tri_v), 2);

        let (tris_u, split_segments) = tri_u.split_transparent(&tri_v);
        assert_eq!(tris_u.len(), 7);

        println!("Split segments: {:?}", split_segments);

        let tris_v = tri_v.split(&split_segments);
        assert_eq!(tris_v.len(), 5);
    }
}
