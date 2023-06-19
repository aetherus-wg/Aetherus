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

#[cfg(test)]
mod tests {
    use crate::{math::Dir3, sim::Attribute};
    use super::*;

    /// A quick test that the constructor works as expected.
    #[test]
    fn test_new_hit() {
        let tag = Attribute::Mirror(0.5);
        let dist = 1.0;
        let side = Side::Inside(Dir3::new(0.0, 0.0, 1.0));
        let hit = Hit::new(&tag, dist, side.clone());
        assert_eq!(*hit.tag(), tag);
        assert_eq!(hit.dist(), dist);
        assert_eq!(*hit.side(), side);
    }

    /// It is unphysical for a hit to be constructed with a distance of zero.
    /// So let's check that the constructor panics in this case. 
    #[test]
    #[should_panic]
    fn test_new_hit_zero_dist() {
        let tag = Attribute::Mirror(0.5);
        let dist = 0.0;
        let side = Side::Inside(Dir3::new(0.0, 0.0, 1.0));
        let _hit = Hit::new(&tag, dist, side);
    }

    /// A test that checks the accessors and mutators for the distance.
    #[test]
    fn test_dist() {
        let tag = Attribute::Mirror(0.5);
        let dist = 1.0;
        let side = Side::Inside(Dir3::new(0.0, 0.0, 1.0));
        let mut hit = Hit::new(&tag, dist, side.clone());
        assert_eq!(hit.dist(), dist);
        
        // Now mutate the value and check that it correctly updates.
        *hit.dist_mut() = 2.0;
        assert_eq!(hit.dist(), 2.0);
    }

    /// A test that checks the accessors for the side.
    /// Note that there is no mutator for the side, as it is immutable.
    #[test]
    fn test_side() {
        let tag = Attribute::Mirror(0.5);
        let dist = 1.0;
        let side = Side::Inside(Dir3::new(0.0, 0.0, 1.0));
        let hit = Hit::new(&tag, dist, side.clone());
        assert_eq!(*hit.side(), side);
    }

    /// A test that checks the accessors for the tag.
    /// Note that there is no mutator for the tag, as it is immutable.
    #[test]
    fn test_tag() {
        let tag = Attribute::Mirror(0.5);
        let dist = 1.0;
        let side = Side::Inside(Dir3::new(0.0, 0.0, 1.0));
        let hit = Hit::new(&tag, dist, side.clone());
        assert_eq!(*hit.tag(), tag);
    }
}