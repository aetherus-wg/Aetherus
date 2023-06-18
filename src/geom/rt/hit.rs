//! Hit struct.

use crate::{access, clone, geom::Side};

/// # Hit
///
/// This is the main struct that is returned from the hit-scan system that is used
/// for ray tracing photon packets in the simulation. This contains information about
/// the hit, including the distance to the hit and the side which is being hit.
#[derive(Clone, PartialEq, Debug)]
pub struct Hit<'a, T> {
    /// Tag reference.
    tag: &'a T,
    /// Distance to the hit.
    dist: f64,
    /// Normal of the surface.
    side: Side,
}

impl<'a, T> Hit<'a, T> {
    access!(tag: T);
    clone!(dist, dist_mut: f64);
    access!(side: Side);

    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(tag: &'a T, dist: f64, side: Side) -> Self {
        debug_assert!(dist > 0.0);

        Self { tag, dist, side }
    }
}
