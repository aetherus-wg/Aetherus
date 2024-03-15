//! Optical material.

use crate::{
    geom::{Emit, Grid, Mesh, Ray},
    math::{rand_isotropic_dir, Dir3, Point3, SphericalCdf, Trans3},
    tools::linear_to_three_dim,
};
use ndarray::Array3;
use rand::Rng;
use std::{
    f64::consts::PI,
    fmt::{Display, Error, Formatter},
};

/// Ray emission structure.
#[derive(Clone)]
pub enum Emitter {
    /// Single beam.
    Beam(Ray),
    /// Points.
    Points(Vec<Point3>),
    /// Weighted points.
    WeightedPoints(Vec<Point3>, Vec<f64>),
    /// Surface mesh.
    Surface(Mesh),
    /// Volume map.
    Volume(Array3<f64>, Grid),
    /// Non-isotropic point source.
    NonIsotropic(SphericalCdf, Trans3),
}

impl Emitter {
    /// Construct a new beam instance.
    #[inline]
    #[must_use]
    pub const fn new_beam(ray: Ray) -> Self {
        Self::Beam(ray)
    }

    /// Construct a new points instance.
    #[inline]
    #[must_use]
    pub fn new_points(points: Vec<Point3>) -> Self {
        debug_assert!(!points.is_empty());

        Self::Points(points)
    }

    /// Construct a new points instance.
    #[inline]
    #[must_use]
    pub fn new_weighted_points(points: Vec<Point3>, weights: &[f64]) -> Self {
        debug_assert!(!points.is_empty());
        debug_assert!(points.len() == weights.len());

        let sum: f64 = weights.iter().sum();
        let mut cumulative_weight = Vec::with_capacity(weights.len());
        let mut total = 0.0;
        for w in weights {
            total += w;
            cumulative_weight.push(total / sum);
        }

        Self::WeightedPoints(points, cumulative_weight)
    }

    /// Construct a new surface instance.
    #[inline]
    #[must_use]
    pub const fn new_surface(mesh: Mesh) -> Self {
        Self::Surface(mesh)
    }

    /// Construct a new volume instance.
    #[inline]
    #[must_use]
    pub fn new_volume(map: Array3<f64>, grid: Grid) -> Self {
        debug_assert!(map.sum() > 0.0);
        debug_assert!(!map.is_empty());

        Self::Volume(map, grid)
    }

    /// Construct a new non-isotropic point source instance.
    #[inline]
    #[must_use]
    pub fn new_non_isotropic(cdf: SphericalCdf, trans: Trans3) -> Self {
        Self::NonIsotropic(cdf, trans)
    }

    /// Emit a new ray.
    #[inline]
    #[must_use]
    pub fn emit<R: Rng>(&self, rng: &mut R) -> Ray {
        match *self {
            Self::Beam(ref ray) => ray.clone(),
            Self::Points(ref ps) => {
                Ray::new(ps[rng.gen_range(0..ps.len())], rand_isotropic_dir(rng))
            }
            Self::WeightedPoints(ref ps, ref ws) => {
                let r: f64 = rng.gen();
                for (p, w) in ps.iter().zip(ws) {
                    if r <= *w {
                        return Ray::new(*p, rand_isotropic_dir(rng));
                    }
                }
                unreachable!("Failed to determine weighted point to emit from.");
            }
            Self::Surface(ref mesh) => mesh.cast(rng),
            Self::Volume(ref map, ref grid) => {
                let r = rng.gen_range(0.0..map.sum());
                let mut total = 0.0;
                for n in 0..map.len() {
                    let index = linear_to_three_dim(n, grid.res());
                    total += map[index];
                    if total >= r {
                        let pos = grid.gen_voxel(&index).rand_pos(rng);
                        let dir = rand_isotropic_dir(rng);
                        return Ray::new(pos, dir);
                    }
                }
                panic!("Failed to emit ray from volume.")
            }
            Self::NonIsotropic(ref cdf, ref trans) => {
                // Using the logic from the isotropic emission case.
                let (az, pol) = cdf.sample(rng);

                // Note that we have to invert the polar angle, as this is taken from
                // vertically up, whereas the light intensities assume vertically down.
                let x = az.cos() * (PI - pol).sin();
                let y = az.sin() * (PI - pol).sin();
                let z = (PI - pol).cos();

                let dir = trans.transform_vector(&Dir3::new(x, y, z).data());

                Ray::new(
                    trans
                        .transform_point(&Point3::new(0.0, 0.0, 0.0).data())
                        .into(),
                    dir.into(),
                )
            }
        }
    }
}

impl Display for Emitter {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        let kind = match *self {
            Self::Beam { .. } => "Beam",
            Self::Points { .. } => "Points",
            Self::WeightedPoints { .. } => "WeightedPoints",
            Self::Surface { .. } => "Surface",
            Self::Volume { .. } => "Volume",
            Self::NonIsotropic { .. } => "Non-isotropic",
        };
        write!(fmt, "{}", kind)
    }
}

#[cfg(test)]
mod tests {
    use super::Emitter;
    use rand;
    use assert_approx_eq::assert_approx_eq;
    use crate::{
        geom::{Ray, Mesh, SmoothTriangle, Triangle}, 
        diag::Average,
        math::{Point3, Dir3},
    };

    #[test]
    fn test_beam_emitter() {
        let mut rng = rand::thread_rng();
        let emit_ray = Ray::new(Point3::new(0.0, 0.0, 0.0), Dir3::new(1.0, 0.0, 0.0));
        let emitter = Emitter::new_beam(emit_ray.clone());

        let emitted_ray = emitter.emit(&mut rng);
        assert_eq!(emitted_ray.dir(), emit_ray.dir());
        assert_eq!(emitted_ray.pos(), emit_ray.pos());
    }

    #[test]
    fn test_points_emitter() {
        let points = vec![Point3::new(1.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.0), Point3::new(0.0, 0.0, 1.0)];
        let emitter = Emitter::new_points(points.clone());

        let mut ave_x = Average::new();
        let mut ave_y = Average::new();
        let mut ave_z = Average::new();
        let mut rng = rand::thread_rng();
        for _ in 0..100_000 {
            let emitted_ray = emitter.emit(&mut rng);
            assert!(points.contains(emitted_ray.pos()));
            ave_x += emitted_ray.dir().x();
            ave_y += emitted_ray.dir().y();
            ave_z += emitted_ray.dir().z();
        }

        // In the case that the emission is isotropic, this should even out to be about zero.
        // I'm testing this to within 1% here. 
        assert_approx_eq!(ave_x.ave(), 0.0, 0.01);
        assert_approx_eq!(ave_y.ave(), 0.0, 0.01);
        assert_approx_eq!(ave_z.ave(), 0.0, 0.01);
    }

    #[test]
    fn test_weighted_points_emitter() {
        let points = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)];
        let weights = vec![1.0, 2.0];
        let emitter = Emitter::new_weighted_points(points.clone(), &weights);

        let mut ave_x = Average::new();
        let mut ave_y = Average::new();
        let mut ave_z = Average::new();
        let mut ave_dir_x = Average::new();
        let mut ave_dir_y = Average::new();
        let mut ave_dir_z = Average::new();
        let mut rng = rand::thread_rng();
        for _ in 0..100_000 {
            let emitted_ray = emitter.emit(&mut rng);
            
            ave_x += emitted_ray.pos().x();
            ave_y += emitted_ray.pos().y();
            ave_z += emitted_ray.pos().z();

            ave_dir_x += emitted_ray.dir().x();
            ave_dir_y += emitted_ray.dir().y();
            ave_dir_z += emitted_ray.dir().z();
        }

        assert_approx_eq!(ave_x.ave(), 0.666, 0.005);
        assert_eq!(ave_y.ave(), 0.0);
        assert_eq!(ave_z.ave(), 0.0);

        // In the case that the emission is isotropic, this should even out to be about zero.
        // I'm testing this to within 1% here. 
        assert_approx_eq!(ave_dir_x.ave(), 0.0, 0.01);
        assert_approx_eq!(ave_dir_y.ave(), 0.0, 0.01);
        assert_approx_eq!(ave_dir_z.ave(), 0.0, 0.01);
    }

    #[test]
    fn test_surface_emitter() {
        let mut rng = rand::thread_rng();
        let norm = Dir3::new(0.0, 0.0, 1.0);

        // Make a single upward facing triangle to emit from. 
        let triangles = vec![ SmoothTriangle::new(
            Triangle::new([
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(1.0, 0.0, 0.0),
                Point3::new(0.0, 1.0, 0.0),
        ]),
            [Dir3::new(0.0, 0.0, 1.0), Dir3::new(0.0, 0.0, 1.0), Dir3::new(0.0, 0.0, 1.0)],
        )];
        let mesh = Mesh::new(triangles);
        let emitter = Emitter::new_surface(mesh);

        let emitted_ray = emitter.emit(&mut rng);
        assert_eq!(emitted_ray.dir(), &norm);
        assert_eq!(emitted_ray.pos().z(), 0.0);
        assert!(emitted_ray.pos().x() >= 0.0 && emitted_ray.pos().x() <= 1.0);
        assert!(emitted_ray.pos().y() >= 0.0 && emitted_ray.pos().y() <= 1.0);
    }
}