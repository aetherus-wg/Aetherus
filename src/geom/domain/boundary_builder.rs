use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use arctk_attr::file;
use crate::{
    fmt_report,
    geom::{Boundary, BoundaryCondition, Cube},
    math::Vec3, 
    phys::{ReflectanceBuilder, ReflectanceBuilderShim},
};

#[file]
#[derive(Serialize)]
pub struct BoundaryBuilder {
    boundary: (Vec3, Vec3),
    top: Option<BoundaryConditionBuilder>,
    bottom: Option<BoundaryConditionBuilder>,
    north: Option<BoundaryConditionBuilder>,
    east: Option<BoundaryConditionBuilder>,
    south: Option<BoundaryConditionBuilder>,
    west: Option<BoundaryConditionBuilder>,
}

impl BoundaryBuilder {
    pub fn build(&self) -> Boundary {
        let bounding_box = Cube::new(self.boundary.0.data().into(), self.boundary.1.data().into());
        let top = match &self.top {
            Some(a) => a.build(),
            None => BoundaryCondition::default(),
        };
        let bottom: BoundaryCondition = match &self.bottom {
            Some(a) => a.build(),
            None => BoundaryCondition::default(),
        };
        let north: BoundaryCondition = match &self.north {
            Some(a) => a.build(),
            None => BoundaryCondition::default(),
        };
        let south = match &self.south {
            Some(a) => a.build(),
            None => BoundaryCondition::default(),
        };
        let east = match &self.east {
            Some(a) => a.build(),
            None => BoundaryCondition::default(),
        };
        let west = match &self.west {
            Some(a) => a.build(),
            None => BoundaryCondition::default(),
        };

        Boundary {
            bounding_box, 
            top,
            bottom,
            north,
            south,
            east,
            west,
        }
    }
}

impl Display for BoundaryBuilder {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(fmt, "...")?;
        fmt_report!(fmt, "...", "boundary");
        fmt_report!(fmt, format!("[{}, {}", self.boundary.0.x(), self.boundary.0.y()), "mins");
        fmt_report!(fmt, format!("[{}, {}", self.boundary.0.x(), self.boundary.0.y()), "maxs");
        
        match &self.top {
            Some(a) => fmt_report!(fmt, a, "top"),
            None => fmt_report!(fmt, "none", "top"),
        };

        match &self.bottom {
            Some(a) => fmt_report!(fmt, a, "bottom"),
            None => fmt_report!(fmt, "none", "bottom"),
        };
        
        match &self.north {
            Some(a) => fmt_report!(fmt, a, "north"),
            None => fmt_report!(fmt, "none", "north"),
        };

        match &self.south {
            Some(a) => fmt_report!(fmt, a, "south"),
            None => fmt_report!(fmt, "none", "south"),
        };

        match &self.east {
            Some(a) => fmt_report!(fmt, a, "east"),
            None => fmt_report!(fmt, "none", "east"),
        };

        match &self.west {
            Some(a) => fmt_report!(fmt, a, "west"),
            None => fmt_report!(fmt, "none", "west"),
        };

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BoundaryConditionBuilder {
    Kill,
    Reflect(ReflectanceBuilderShim),
    Periodic(f64),
    #[cfg(feature = "mpi")]
    MpiRank(usize),
}

impl BoundaryConditionBuilder {
    pub fn build(&self) -> BoundaryCondition {
        match self {
            Self::Kill => BoundaryCondition::Kill,
            Self::Periodic(dist) => BoundaryCondition::Periodic(dist.clone()),
            Self::Reflect(ref_shim) => {
                let ref_build: ReflectanceBuilder = ref_shim.clone().into();
                let ref_model = ref_build.build().expect("Unable to load reflectance model for boundary. ");
                BoundaryCondition::Reflect(ref_model)
            }
        }
    }
}


impl Display for BoundaryConditionBuilder {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Kill => {
                writeln!(fmt, "Kill: ...")?;
                Ok(())
            }
            Self::Reflect(ref reflectance) => {
                writeln!(fmt, "Reflector: ...")?;
                fmt_report!(fmt, format!("{:?}, {:?}, {:?}", reflectance.0, reflectance.1, reflectance.2), "reflectance");
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
