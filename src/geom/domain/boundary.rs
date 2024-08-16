use crate::{
    access, clone, fmt_report, 
    geom::{plane::ray_plane_intersection, Cube, Hit, Ray, Side, Trace}, 
    math::{Dir3, Point3, Vec3}, 
    phys::{Photon, Reflectance}, 
    sim::Attribute,
    ord::cartesian::{X, Y, Z},
};
use rand::rngs::ThreadRng;
use std::fmt::{Display, Formatter};

/// Struct that represents a boundary.
/// This will be used to determine how the boundary conditions behaves when it interacts
/// with photon packets.
#[derive(Clone)]
pub struct Boundary {
    pub bounding_box: Cube,
    pub top: BoundaryCondition,
    pub bottom: BoundaryCondition,
    pub north: BoundaryCondition,
    pub east: BoundaryCondition,
    pub south: BoundaryCondition,
    pub west: BoundaryCondition,
}

impl Boundary {
    access!(bounding_box, bounding_box_mut: Cube);
    access!(top, top_mut: BoundaryCondition);
    access!(bottom, bottom_mut: BoundaryCondition);
    access!(north, north_mut: BoundaryCondition);
    access!(east, east_mut: BoundaryCondition);
    access!(south, south_mut: BoundaryCondition);
    access!(west, west_mut: BoundaryCondition);

    pub fn new(bounding_box: Cube) -> Self {
        Self {
            bounding_box,
            top: BoundaryCondition::default(),
            bottom: BoundaryCondition::default(),
            north: BoundaryCondition::default(),
            east: BoundaryCondition::default(),
            south: BoundaryCondition::default(),
            west: BoundaryCondition::default(),
        }
    }

    pub fn new_kill(bounding_box: Cube) -> Self {
        Self {
            bounding_box,
            top: BoundaryCondition::Kill,
            bottom: BoundaryCondition::Kill,
            north: BoundaryCondition::Kill,
            east: BoundaryCondition::Kill,
            south: BoundaryCondition::Kill,
            west: BoundaryCondition::Kill,
        }
    }

    pub fn new_reflect(bounding_box: Cube, reflect: Reflectance) -> Self {
        Self {
            bounding_box,
            top: BoundaryCondition::Reflect(reflect.clone()),
            bottom: BoundaryCondition::Reflect(reflect.clone()),
            north: BoundaryCondition::Reflect(reflect.clone()),
            east: BoundaryCondition::Reflect(reflect.clone()),
            south: BoundaryCondition::Reflect(reflect.clone()),
            west: BoundaryCondition::Reflect(reflect.clone()),
        }
    }

    pub fn new_periodic(bounding_box: Cube, padding: f64) -> Self {
        Self {
            bounding_box,
            top: BoundaryCondition::Periodic(padding),
            bottom: BoundaryCondition::Periodic(padding),
            north: BoundaryCondition::Periodic(padding),
            east: BoundaryCondition::Periodic(padding),
            south: BoundaryCondition::Periodic(padding),
            west: BoundaryCondition::Periodic(padding),
        }
    }

    #[inline]
    pub fn apply<'a>(&self, rng: &mut ThreadRng, hit: &'a BoundaryHit<'a>, phot: &mut Photon) {
        match hit.condition() {
            BoundaryCondition::Kill => {
                // Handle Kill variant
                phot.kill();
            }
            BoundaryCondition::Reflect(reflectance) => {
                // Handle Reflect variant
                match reflectance.reflect(rng, &phot, &hit.get_hit()) {
                    Some(ray) => *phot.ray_mut() = ray,
                    None => phot.kill(),
                }
            }
            #[cfg(not(feature = "mpi"))]
            BoundaryCondition::Periodic(padding) => {
                // Get the opposing boundary
                self.set_ray_to_opposite_boundary(&mut phot.ray_mut(), hit.direction(), padding);
            }
            #[cfg(feature = "mpi")]
            BoundaryCondition::Periodic(padding) => {
                // Handle this variant in the case of MPI.
                unimplemented!()
            }
            #[cfg(feature = "mpi")]
            BoundaryCondition::MpiRank => {
                // Handle MpiRank variant
            }
        };
    }

    /// Provides the translation to a Point3 (+/- a padding) to move it from one
    /// boundary to the opposing boundary. The primary intended use of this code is
    /// in the application of a periodic boundary on a single compute node.
    #[inline]
    pub fn get_periodic_translation(&self, bound: &BoundaryDirection, padding: &f64) -> Vec3 {
        // First determine the vector component that we need to translate.
        let trans_vec = match bound {
            BoundaryDirection::Top | BoundaryDirection::Bottom => Vec3::new(
                0.0,
                0.0,
                self.bounding_box.maxs()[2] - self.bounding_box.mins()[2] - padding,
            ),
            BoundaryDirection::North | BoundaryDirection::South => Vec3::new(
                0.0,
                self.bounding_box.maxs()[1] - self.bounding_box.mins()[1] - padding,
                0.0,
            ),
            BoundaryDirection::East | BoundaryDirection::West => Vec3::new(
                self.bounding_box.maxs()[0] - self.bounding_box.mins()[0] - padding,
                0.0,
                0.0,
            ),
        };

        // Finally determine the direction of the resulting translation.
        match bound {
            BoundaryDirection::Top | BoundaryDirection::East | BoundaryDirection::North => {
                -trans_vec
            }
            _ => trans_vec,
        }
    }

    #[inline]
    pub fn set_ray_to_opposite_boundary(
        &self,
        ray: &mut Ray,
        bound: &BoundaryDirection,
        padding: &f64,
    ) {
        // First determine the axis that we need to translate.
        let axis = match bound {
            BoundaryDirection::Top | BoundaryDirection::Bottom => 2,
            BoundaryDirection::North | BoundaryDirection::South => 1,
            BoundaryDirection::East | BoundaryDirection::West => 0,
        };

        // Finally, set the position of the ray to the opposite boundary, +/- padding.
        match bound {
            BoundaryDirection::Top | BoundaryDirection::East | BoundaryDirection::North => {
                ray.pos_mut()[axis] = self.bounding_box.mins()[axis] + padding;
            }
            _ => {
                ray.pos_mut()[axis] = self.bounding_box.maxs()[axis] - padding;
            }
        }
    }

    pub fn dist_boundary(&self, ray: &Ray) -> Option<BoundaryHit> {
        if let Some((dist, _side)) = self.bounding_box.dist_side(ray) {
            // Now we have to find the boundary that the ray is going to hit.
            // We can do this by finding the max absolutel component value of the
            // vector. Then, find the dir
            let dir = self.boundary_direction(&ray).expect("Ray outside of boundary. ");
            return Some(BoundaryHit::new(
                self.condition_for_boundary(&dir), 
                dist, 
                dir,
            ))
        }

        None
    }

    #[inline]
    pub fn condition_for_boundary(&self, dir: &BoundaryDirection) -> &BoundaryCondition {
        match &dir {
            BoundaryDirection::Bottom => self.bottom(),
            BoundaryDirection::Top => self.top(),
            BoundaryDirection::North => self.north(),
            BoundaryDirection::South => self.south(),
            BoundaryDirection::West => self.west(),
            BoundaryDirection::East => self.east(),
        }
    }

    #[inline]
    pub fn contains(&self, p: &Point3) -> bool {
        self.bounding_box.contains(p)
    }

    pub fn boundary_direction(&self, ray: &Ray) -> Option<BoundaryDirection> {
        const TOLLERANCE: f64 = 1E-10;
        let dirs = [
            BoundaryDirection::Bottom,
            BoundaryDirection::Top,
            BoundaryDirection::South,
            BoundaryDirection::North,
            BoundaryDirection::West,
            BoundaryDirection::East,
        ];
        
        let min_point = Point3::new(self.bounding_box().mins()[0], self.bounding_box().mins()[1], self.bounding_box().mins()[2]);
        let max_point = Point3::new(self.bounding_box().maxs()[0], self.bounding_box().maxs()[1], self.bounding_box().maxs()[2]);
        let mut facing_dir = None;
        for (i, &dir) in dirs.iter().enumerate() {
            if let Some(point) = ray_plane_intersection(&ray, if i % 2 == 0 { min_point } else { max_point }, dir.normal_vector()) {
                if point.x() >= min_point.x() - TOLLERANCE  && point.x() <= max_point.x() + TOLLERANCE &&
                   point.y() >= min_point.y() - TOLLERANCE  && point.y() <= max_point.y() + TOLLERANCE &&
                   point.z() >= min_point.z() - TOLLERANCE && point.z() <= max_point.z() + TOLLERANCE 
                {
                    facing_dir = Some(dir.clone());
                    break;
                }
            }
        }

        return facing_dir;
    }

    /// If the given position is contained within the grid,
    /// generate the index for the given position within the grid.
    #[inline]
    #[must_use]
    pub fn gen_index(&self, p: &Point3, res: [usize; 3]) -> Option<[usize; 3]> {
        self.contains(p).then(|| {
            let mins = self.bounding_box.mins();
            let maxs = self.bounding_box.maxs();

            [
                (((p.x() - mins.x()) / (maxs.x() - mins.x())) * res[X] as f64).floor()
                    as usize,
                (((p.y() - mins.y()) / (maxs.y() - mins.y())) * res[Y] as f64).floor()
                    as usize,
                (((p.z() - mins.z()) / (maxs.z() - mins.z())) * res[Z] as f64).floor()
                    as usize,
            ]
        })
    }
}

impl Display for Boundary {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fmt_report!(f, self.bounding_box, "bounding box");
        fmt_report!(f, self.top, "top");
        fmt_report!(f, self.bottom, "bottom");
        fmt_report!(f, self.north, "north");
        fmt_report!(f, self.east, "east");
        fmt_report!(f, self.south, "south");
        fmt_report!(f, self.west, "west");
        Ok(())
    }
}

/// Describing a boundary at which the action is triggered.
/// This will help determine how the boundary conditions behaves when it interacts
/// with photon packets.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum BoundaryDirection {
    /// The boundary at the maximum z-value.
    Top,
    /// The boundary at the minimum z-value.
    Bottom,
    /// The boundary at the maximum y-value.
    North,
    /// The boundary at the maximum x-value.
    East,
    /// The boundary at the minimum y-value.
    South,
    /// The boundary at the minimum y-value.
    West,
}

impl BoundaryDirection {
    pub fn opposing(&self) -> BoundaryDirection {
        match self {
            BoundaryDirection::Top => BoundaryDirection::Bottom,
            BoundaryDirection::Bottom => BoundaryDirection::Top,
            BoundaryDirection::North => BoundaryDirection::South,
            BoundaryDirection::South => BoundaryDirection::North,
            BoundaryDirection::East => BoundaryDirection::West,
            BoundaryDirection::West => BoundaryDirection::East,
        }
    }

    pub fn normal_vector(&self) -> Dir3 {
        match self {
            BoundaryDirection::Top => Dir3::new(0.0, 0.0, -1.0),
            BoundaryDirection::Bottom => Dir3::new(0.0, 0.0, 1.0),
            BoundaryDirection::North => Dir3::new(0.0, -1.0, 0.0),
            BoundaryDirection::South => Dir3::new(0.0, 1.0, 0.0),
            BoundaryDirection::East => Dir3::new(-1.0, 0.0, 0.0),
            BoundaryDirection::West => Dir3::new(1.0, 0.0, 0.0),
        }
    }
}

impl Display for BoundaryDirection {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            BoundaryDirection::Top => write!(f, "Top"),
            BoundaryDirection::Bottom => write!(f, "Bottom"),
            BoundaryDirection::North => write!(f, "North"),
            BoundaryDirection::East => write!(f, "East"),
            BoundaryDirection::South => write!(f, "South"),
            BoundaryDirection::West => write!(f, "West"),
        }
    }
}

#[derive(Default, Debug, PartialEq, Clone)]
pub enum BoundaryCondition {
    /// Any photon packet that intersects with this boundary will be down-weighted
    /// and removed from the simulation.
    #[default]
    Kill,
    /// Any photon packet that intersects with this boundarty will be specularly
    /// reflected back into the domain.
    Reflect(Reflectance),
    /// Any photon that intersects with this boundary will be transferred to the
    /// opposing boundary and re-emitted
    /// The number defines the padding distance from the oppising edge (to avoid instant re-collision).
    Periodic(f64),
    /// Photons that intersect this boundary will be collected, buffered and
    /// transferred to the adjacent MPI rank.
    #[cfg(feature = "mpi")]
    MpiRank(usize),
}


impl Display for BoundaryCondition {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Kill => {
                writeln!(fmt, "Kill: ...")?;
                Ok(())
            }
            Self::Reflect(ref reflectance) => {
                writeln!(fmt, "Reflector: ...")?;
                fmt_report!(fmt, reflectance, "reflectance");
                Ok(())
            },
            Self::Periodic(padding) => {
                writeln!(fmt, "Periodic: ...")?;
                fmt_report!(fmt, padding, "padding");
                Ok(())
            },
            #[cfg(feature = "mpi")]
            Self::MpiRank(rank) => {
                writeln!(fmt, "MPI Rank Transfer: ...")?;
                fmt_report!(fmt, padding, "destination rank");
                Ok(())
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct BoundaryHit<'a> {
    condition: &'a BoundaryCondition,
    dist: f64,
    direction: BoundaryDirection,
}

impl<'a> BoundaryHit<'a> {
    access!(condition: BoundaryCondition);
    clone!(dist, dist_mut: f64);
    access!(direction: BoundaryDirection);

    #[inline]
    #[must_use]
    pub fn new(condition: &'a BoundaryCondition, dist: f64, direction: BoundaryDirection) -> Self {
        debug_assert!(dist > 0.0);
        Self {
            condition,
            dist,
            direction,
        }
    }

    pub fn get_hit(&self) -> Hit<'_, Attribute> {
        Hit::new(
            &Attribute::Mirror(0.0),
            self.dist(),
            Side::Inside(self.direction().normal_vector()),
        )
    }
}

impl<'a> Into<Hit<'a, Attribute<'a>>> for BoundaryHit<'a> {
    fn into(self) -> Hit<'a, Attribute<'a>> {
        // Not the most elegant implementation, as the tag is not used.
        Hit::new(
            &Attribute::Mirror(0.0),
            self.dist(),
            Side::Inside(self.direction().normal_vector()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::{Dir3, Point3};

    #[test]
    fn test_boundary_facing() {
        let boundary = Boundary {
            bounding_box: Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(6.0, 8.0, 10.0)),
            top: BoundaryCondition::default(),
            bottom: BoundaryCondition::default(),
            north: BoundaryCondition::default(),
            east: BoundaryCondition::default(),
            south: BoundaryCondition::default(),
            west: BoundaryCondition::default(),
        };

        // Basic - Check a ray facing zenith.
        let ray = Ray::new(Point3::new(5.0, 5.0, 5.0), Dir3::new(0.0, 0.0, 1.0));
        let boundary_hit = boundary
            .dist_boundary(&ray)
            .expect("Ray not contained within domain. ");

        // Now do our first test.
        assert_eq!(boundary_hit.dist(), 5.0);
        assert_eq!(*boundary_hit.direction(), BoundaryDirection::Top);

        // Update the ray to face nadir.
        let ray = Ray::new(Point3::new(5.0, 5.0, 5.0), Dir3::new(0.0, 0.0, -1.0));
        let boundary_hit = boundary
            .dist_boundary(&ray)
            .expect("Ray not contained within domain. ");
        // Now test that we get the correct boundary.
        assert_eq!(boundary_hit.dist(), 5.0);
        assert_eq!(*boundary_hit.direction(), BoundaryDirection::Bottom);

        // Now do a test for the east boundary.
        // Update the ray to face east.
        let ray = Ray::new(Point3::new(5.0, 5.0, 5.0), Dir3::new(1.0, 0.0, 0.0));
        let boundary_hit = boundary
            .dist_boundary(&ray)
            .expect("Ray not contained within domain. ");
        // Now test that we get the correct boundary.
        assert_eq!(boundary_hit.dist(), 1.0);
        assert_eq!(*boundary_hit.direction(), BoundaryDirection::East);

        // Now do a test for the west boundary.
        // Update the ray to face west.
        let ray = Ray::new(Point3::new(5.0, 5.0, 5.0), Dir3::new(-1.0, 0.0, 0.0));
        let boundary_hit = boundary
            .dist_boundary(&ray)
            .expect("Ray not contained within domain. ");
        // Now test that we get the correct boundary.
        assert_eq!(boundary_hit.dist(), 5.0);
        assert_eq!(*boundary_hit.direction(), BoundaryDirection::West);

        // Now do a test for the north boundary.
        // Update the ray to face north.
        let ray = Ray::new(Point3::new(5.0, 5.0, 5.0), Dir3::new(0.0, 1.0, 0.0));
        let boundary_hit = boundary
            .dist_boundary(&ray)
            .expect("Ray not contained within domain. ");
        // Now test that we get the correct boundary.
        assert_eq!(boundary_hit.dist(), 3.0);
        assert_eq!(*boundary_hit.direction(), BoundaryDirection::North);

        // Now do a test for the south boundary.
        // Update the ray to face south.
        let ray = Ray::new(Point3::new(5.0, 5.0, 5.0), Dir3::new(0.0, -1.0, 0.0));
        let boundary_hit = boundary
            .dist_boundary(&ray)
            .expect("Ray not contained within domain. ");
        // Now test that we get the correct boundary.
        assert_eq!(boundary_hit.dist(), 5.0);
        assert_eq!(*boundary_hit.direction(), BoundaryDirection::South);

        // Now let's do a test that checks an odd combination of directions.
        let ray = Ray::new(Point3::new(5.0, 5.0, 5.0), Dir3::new(1.0, 3.0, 5.0));
        let boundary_hit = boundary
            .dist_boundary(&ray)
            .expect("Ray not contained within domain. ");
        // Now test that we get the correct boundary.

        assert_eq!(
            boundary_hit.dist(),
            (1.0_f64 + 3.0_f64 * 3.0_f64 + 5.0_f64 * 5.0_f64).sqrt()
        );
        assert_eq!(*boundary_hit.direction(), BoundaryDirection::Top);
    }

    #[test]
    fn test_periodic_boundary() {
        let mut rng = rand::thread_rng();

        // Setup a basic boundary to the simulation.
        // Each side is a different length, and is periodic.
        let boundary = Boundary {
            bounding_box: Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(6.0, 8.0, 10.0)),
            top: BoundaryCondition::Periodic(0.0),
            bottom: BoundaryCondition::Periodic(0.0),
            north: BoundaryCondition::Periodic(0.0),
            east: BoundaryCondition::Periodic(0.0),
            south: BoundaryCondition::Periodic(0.0),
            west: BoundaryCondition::Periodic(0.0),
        };

        // Test with padding of 0.0
        let incoming_ray = Ray::new(Point3::new(5.0, 5.0, 9.98), Dir3::new(0.0, 0.0, 1.0));
        let mut incoming_photon = Photon::new(incoming_ray, 550.0, 1.0);

        let bhit = boundary
            .dist_boundary(incoming_photon.ray())
            .expect("Ray not contained within domain. ");
        boundary.apply(&mut rng, &bhit, &mut incoming_photon);
        assert_eq!(*incoming_photon.ray().pos(), Point3::new(5.0, 5.0, 0.0));
        assert_eq!(*incoming_photon.ray().dir(), Dir3::new(0.0, 0.0, 1.0));

        let boundary = Boundary {
            bounding_box: Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(6.0, 8.0, 10.0)),
            top: BoundaryCondition::Periodic(0.01),
            bottom: BoundaryCondition::Periodic(0.01),
            north: BoundaryCondition::Periodic(0.01),
            east: BoundaryCondition::Periodic(0.01),
            south: BoundaryCondition::Periodic(0.01),
            west: BoundaryCondition::Periodic(0.01),
        };

        // Test with padding of 0.01
        let incoming_ray = Ray::new(Point3::new(5.0, 0.02, 5.0), Dir3::new(0.1, -0.9, 0.0));
        let mut incoming_photon = Photon::new(incoming_ray, 550.0, 1.0);

        let bhit = boundary
            .dist_boundary(incoming_photon.ray())
            .expect("Ray not contained within domain. ");
        boundary.apply(&mut rng, &bhit, &mut incoming_photon);
        assert_eq!(*incoming_photon.ray().pos(), Point3::new(5.0, 7.99, 5.0));
        assert_eq!(*incoming_photon.ray().dir(), Dir3::new(0.1, -0.9, 0.0));
    }

    #[test]
    fn test_cube_boundary_direction() {
        // Setup a basic boundary to the simulation.
        // Each side is a different length, and is periodic.
        let boundary = Boundary {
            bounding_box: Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0)),
            top: BoundaryCondition::Periodic(0.0),
            bottom: BoundaryCondition::Periodic(0.0),
            north: BoundaryCondition::Periodic(0.0),
            east: BoundaryCondition::Periodic(0.0),
            south: BoundaryCondition::Periodic(0.0),
            west: BoundaryCondition::Periodic(0.0),
        };
        
        let incoming_ray = Ray::new(Point3::new(0.5, 0.5, 0.5), Dir3::new(1.0, 0.0, 0.0));
        let b = boundary.boundary_direction(&incoming_ray);
        assert_eq!(b, Some(BoundaryDirection::East));

        let incoming_ray = Ray::new(Point3::new(0.5, 0.5, 0.5), Dir3::new(-1.0, 0.0, 0.0));
        let b = boundary.boundary_direction(&incoming_ray);
        assert_eq!(b, Some(BoundaryDirection::West));

        let incoming_ray = Ray::new(Point3::new(0.5, 0.5, 0.5), Dir3::new(0.0, 1.0, 0.0));
        let b = boundary.boundary_direction(&incoming_ray);
        assert_eq!(b, Some(BoundaryDirection::North));

        let incoming_ray = Ray::new(Point3::new(0.5, 0.5, 0.5), Dir3::new(0.0, -1.0, 0.0));
        let b = boundary.boundary_direction(&incoming_ray);
        assert_eq!(b, Some(BoundaryDirection::South));

        let incoming_ray = Ray::new(Point3::new(0.5, 0.5, 0.5), Dir3::new(0.0, 0.0, 1.0));
        let b = boundary.boundary_direction(&incoming_ray);
        assert_eq!(b, Some(BoundaryDirection::Top));

        let incoming_ray = Ray::new(Point3::new(0.5, 0.5, 0.5), Dir3::new(0.0, 0.0, -1.0));
        let b = boundary.boundary_direction(&incoming_ray);
        assert_eq!(b, Some(BoundaryDirection::Bottom));
    }

    #[test]
    fn test_boundary_edge() {
        // Setup a basic boundary to the simulation.
        // Each side is a different length, and is periodic.
        let boundary = Boundary {
            bounding_box: Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0)),
            top: BoundaryCondition::Periodic(0.0),
            bottom: BoundaryCondition::Periodic(0.0),
            north: BoundaryCondition::Periodic(0.0),
            east: BoundaryCondition::Periodic(0.0),
            south: BoundaryCondition::Periodic(0.0),
            west: BoundaryCondition::Periodic(0.0),
        };
        
        let incoming_ray = Ray::new(Point3::new(1.0, 0.5, 0.5), Dir3::new(1.0, 0.0, 0.0));
        let b = boundary.boundary_direction(&incoming_ray);
        assert_eq!(b, Some(BoundaryDirection::East));
    }
}
