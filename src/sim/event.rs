//! Event enumeration.

use crate::geom::Hit;

/// Event determination enumeration.
#[derive(PartialEq, Debug)]
pub enum Event<'a, T> 
{
    /// Voxel boundary collision.
    Voxel(f64),
    /// Scattering event.
    Scattering(f64),
    /// Surface hit.
    Surface(Hit<'a, T>),
}

impl<'a, T> Event<'a, T> {
    /// Construct a new instance.
    /// Surface interactions are prioritised, then boundary collisions, and finally scattering events.
    #[inline]
    #[must_use]
    pub fn new(
        voxel_dist: f64,
        scat_dist: f64,
        surf_hit: Option<Hit<'a, T>>,
        bump_dist: f64,
    ) -> Self {
        debug_assert!(voxel_dist > 0.0);
        debug_assert!(scat_dist > 0.0);
        debug_assert!(bump_dist > 0.0);

        if let Some(hit) = surf_hit {
            if voxel_dist < (hit.dist() + bump_dist) {
                if scat_dist < (voxel_dist + bump_dist) {
                    return Self::Scattering(scat_dist);
                }
                return Self::Voxel(voxel_dist);
            }

            if scat_dist < (hit.dist() + bump_dist) {
                return Self::Scattering(scat_dist);
            }
            return Self::Surface(hit);
        }

        if scat_dist < (voxel_dist + bump_dist) {
            return Self::Scattering(scat_dist);
        }
        Self::Voxel(voxel_dist)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        sim::Attribute,
        geom::Side,
        math::Dir3,
    };
    use super::*;

    /// In this scenario, the surface hit is the closest event. 
    #[test]
    fn test_new_surface_hit() {
        let surf_hit = Some(Hit::new(&Attribute::Mirror(0.5), 1.0, Side::Outside(Dir3::new(1.0, 0.0, 0.0))));
        let event = Event::new(2.0, 3.0, surf_hit, 0.5);

        // Check each of the components of the event.
        if let Event::Surface(hit) = event {
            assert_eq!(hit.tag(), &Attribute::Mirror(0.5));
            assert_eq!(hit.dist(), 1.0);
            assert_eq!(hit.side(), &Side::Outside(Dir3::new(1.0, 0.0, 0.0)));
        } else {
            panic!("Expected surface hit.");
        }
    }

    #[test]
    fn test_new_voxel_collision() {
        let event: Event<'_, Attribute> = Event::new(2.0, 3.0, None, 0.5);
        assert_eq!(event, Event::Voxel(2.0));
    }

    #[test]
    fn test_new_scattering_event() {
        let surf_hit = Some(Hit::new(&Attribute::Mirror(0.5), 2.0, Side::Outside(Dir3::new(1.0, 0.0, 0.0))));
        let event = Event::new(2.0, 1.0, surf_hit, 4.0);
        assert_eq!(event, Event::Scattering(1.0));
    }
}
