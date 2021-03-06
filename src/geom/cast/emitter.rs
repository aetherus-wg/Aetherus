//! Optical material.

use crate::{
    geom::{Emit, Grid, Mesh, Ray},
    math::{rand_isotropic_dir, Dir3, Point3, SphericalCdf},
    tools::linear_to_three_dim,
};
use ndarray::Array3;
use rand::Rng;
use std::{
    f64::consts::PI,
    fmt::{Display, Error, Formatter},
};

/// Ray emission structure.
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
    NonIsotropicPoints(Vec<Point3>, SphericalCdf),
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
    pub fn new_non_isotropic_points(points: Vec<Point3>, cdf: SphericalCdf) -> Self {
        Self::NonIsotropicPoints(points, cdf)
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
            Self::NonIsotropicPoints(ref points, ref cdf) => {
                // Using the logic from the isotropic emission case.
                let (az, pol) = cdf.sample(rng);

                // Note that we have to invert the polar angle, as this is taken from
                // vertically up, whereas the light intensities assume vertically down.
                let x = az.cos() * (PI - pol).sin();
                let y = az.sin() * (PI - pol).sin();
                let z = (PI - pol).cos();

                Ray::new(points[rng.gen_range(0..points.len())], Dir3::new(x, y, z))
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
            Self::NonIsotropicPoints { .. } => "Non-isotropic Points",
        };
        write!(fmt, "{}", kind)
    }
}
