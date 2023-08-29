//! Optical surface structure.

use crate::{access, fmt_report, geom::Mesh};
use std::fmt::{Display, Error, Formatter};

/// Optical surface.
pub struct Surface<'a, T> {
    /// Mesh.
    mesh: Mesh,
    /// Attribute.
    attr: &'a T,
}

impl<'a, T> Surface<'a, T> {
    access!(mesh: Mesh);
    access!(attr: T);

    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub const fn new(mesh: Mesh, attr: &'a T) -> Self {
        Self { mesh, attr }
    }
}

impl<T: Display> Display for Surface<'_, T> {
    #[inline]
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