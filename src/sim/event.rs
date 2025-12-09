//! Event enumeration.

use crate::geom::{BoundaryHit, Hit};

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
    /// Boundary
    Boundary(BoundaryHit<'a>)
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
        boundary_hit: BoundaryHit<'a>,
        bump_dist: f64,
    ) -> Self {
        debug_assert!(voxel_dist > 0.0);
        debug_assert!(scat_dist > 0.0);
        debug_assert!(bump_dist > 0.0);

        // Logically, if there is any geometry, it should be within the octree
        // which is contained within the boundary.
        if let Some(hit) = surf_hit {
            if (voxel_dist + bump_dist) < hit.dist() {
                if scat_dist < (voxel_dist + bump_dist) {
                    return Self::Scattering(scat_dist);
                }
                return Self::Voxel(voxel_dist);
            }

            if (scat_dist + bump_dist) < hit.dist() {
                return Self::Scattering(scat_dist);
            }
            return Self::Surface(hit);
        }

        if boundary_hit.dist() <= (voxel_dist + bump_dist) && boundary_hit.dist()<scat_dist {
            return Self::Boundary(boundary_hit);
        }

        // WARN: Potentially skipping a voxel, which can alter the results for OutputVolume
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
        geom::{Side, BoundaryCondition, BoundaryDirection},
        math::Dir3,
    };
    use super::*;

    /// In this scenario, the surface hit is the closest event.
    #[test]
    fn test_new_surface_hit() {
        let surf_hit = Some(Hit::new(&Attribute::Mirror(0.5), 1.0, Side::Outside(Dir3::new(1.0, 0.0, 0.0))));
        let boundary_hit = BoundaryHit::new(&BoundaryCondition::Kill, f64::INFINITY, BoundaryDirection::Bottom);
        let event = Event::new(2.0, 3.0, surf_hit, boundary_hit, 0.5);

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
        let boundary_hit = BoundaryHit::new(&BoundaryCondition::Kill, f64::INFINITY, BoundaryDirection::Bottom);
        let event: Event<'_, Attribute> = Event::new(2.0, 3.0, None, boundary_hit, 0.5);
        assert_eq!(event, Event::Voxel(2.0));
    }

    #[test]
    fn test_new_scattering_event() {
        let surf_hit = Some(Hit::new(&Attribute::Mirror(0.5), 2.0, Side::Outside(Dir3::new(1.0, 0.0, 0.0))));
        let boundary_hit = BoundaryHit::new(&BoundaryCondition::Kill, f64::INFINITY, BoundaryDirection::Bottom);
        let event = Event::new(2.0, 1.0, surf_hit, boundary_hit, 0.5);
        assert_eq!(event, Event::Scattering(1.0));
    }

    #[test]
    fn test_new_boundary_event() {

        let bhit = BoundaryHit::new(&BoundaryCondition::Periodic(0.0), 0.1, BoundaryDirection::North);
        let event: Event<'_, Attribute<'_>> = Event::new(2.0, 1.0, None, bhit.clone(), 0.5);
        assert_eq!(event, Event::Boundary(bhit));
    }

    // TODO: Add test to check surfaces are given priority for when voxels faces coincide with
    // surfaces, or when bump_dist might cause photons to pass through the surface undetected
}
