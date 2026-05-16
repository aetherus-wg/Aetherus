//! Optical surface structure.

use crate::{access, fmt_report, geom::{Collide, Mesh}};
use std::fmt::{Display, Error, Formatter};

/// Optical surface.
#[derive(Clone)]
pub struct Surface<T> {
    /// Mesh.
    mesh: Mesh,
    /// Object.
    attr: T,
}

impl<T> Surface<T> {

    access!(mesh, mesh_mut: Mesh);
    access!(attr: T);

    /// Construct a new instance.
    #[must_use]
    pub fn new(mesh: Mesh, attr: T) -> Self {
        Self { mesh, attr }
    }
}

impl<T: Display> Display for Surface<T> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        writeln!(fmt, "...")?;
        fmt_report!(fmt, self.mesh, "mesh");
        fmt_report!(fmt, self.attr, "attribute");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        geom::{Mesh, Surface, SmoothTriangle, Triangle},
        math::{Dir3, Point3},
        sim::Attribute,
    };
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_new() {
        // Make a single upward facing triangle for the surface.
        let triangles = vec![ SmoothTriangle::new(
            Triangle::new([
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(1.0, 0.0, 0.0),
                Point3::new(0.0, 1.0, 0.0),
        ]),
            [Dir3::new(0.0, 0.0, 1.0), Dir3::new(0.0, 0.0, 1.0), Dir3::new(0.0, 0.0, 1.0)],
        )];

        let mesh = Mesh::new(triangles);
        let surf = Surface::new(mesh, &Attribute::Mirror(0.5));

        assert_approx_eq!(surf.mesh().area(), 0.5);
    }
}

impl<T> Collide<Surface<T>> for Surface<T> {
    #[inline]
    fn overlap(&self, other: &Surface<T>) -> bool {
        self.mesh.overlap(&other.mesh)
    }
}
