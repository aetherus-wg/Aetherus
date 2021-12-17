//! Transformable trait.

use crate::math::Trans3;

/// A trait that indicates that a type may be transformed.
/// 
/// Any type implementing this trait can be transformed using the 3-dimensional
/// transform in the `math` module (`crate::math::Trans3`).
/// This is currently performed using an `nalgebra::geometry::Similarity3`, which is a
/// uniform scaling, followed by a rotation, followed by a translation. 
pub trait Transformable {
    /// Apply the given transformation.
    fn transform(&mut self, trans: &Trans3);
}

#[cfg(test)]
mod tests {
    // We implement the transformable for the triangle primitive, so we shall use this for tests.
    use super::{Trans3, Transformable};
    use nalgebra::Vector3;
    use std::f64;
    use crate::{
        geom::Triangle,
        math::{Point3, Vec3},
    };

    fn unti_triangle() -> Triangle {
        Triangle::new([
            Point3::new(0., 0., 0.),
            Point3::new(1., 0., 0.),
            Point3::new(1., 1., 0.),
        ])
    }

    #[test]
    fn scale_test() {
        let mut tri = unti_triangle();

        // First Scale up.
        tri.transform(&Trans3::new(Vector3::new(0., 0., 0.), Vector3::new(0., 0., 0.), 2.0));
        assert_eq!(*tri.verts(), [
            Point3::new(0., 0., 0.),
            Point3::new(2., 0., 0.),
            Point3::new(2., 2., 0.),
        ]);

        // Now check it is reversible and scale back down. 
        tri.transform(&Trans3::new(Vector3::new(0., 0., 0.), Vector3::new(0., 0., 0.), 0.25));
        assert_eq!(*tri.verts(), [
            Point3::new(0., 0., 0.),
            Point3::new(0.5, 0., 0.),
            Point3::new(0.5, 0.5, 0.),
        ]);
    }

    #[test]
    fn translate_test() {
        let mut tri = unti_triangle();

        // First Scale up.
        tri.transform(&Trans3::new(Vector3::new(1.5, 1.5, 1.5), Vector3::new(0., 0., 0.), 1.0));
        assert_eq!(*tri.verts(), [
            Point3::new(1.5, 1.5, 1.5),
            Point3::new(2.5, 1.5, 1.5),
            Point3::new(2.5, 2.5, 1.5),
        ]);

        // Now check it is reversible and scale back down. 
        tri.transform(&Trans3::new(Vector3::new(-4., -4., -4.), Vector3::new(0., 0., 0.), 1.0));
        assert_eq!(*tri.verts(), [
            Point3::new(-2.5, -2.5, -2.5),
            Point3::new(-1.5, -2.5, -2.5),
            Point3::new(-1.5, -1.5, -2.5),
        ]);
    }

    #[test]
    fn rotation_test() {
        let mut tri = unti_triangle();

        // Let us rotate around the y Axis by Pi radians (90 degrees). 
        tri.transform(&Trans3::new(Vector3::new(0., 0., 0.), Vector3::y() * f64::consts::FRAC_PI_2, 1.0));
        // Check that the components have correctly transformed into the correct axis. 
        assert_eq!(tri.verts()[1][2], -1.0);
        assert_eq!(tri.verts()[2][2], -1.0);
    }
}