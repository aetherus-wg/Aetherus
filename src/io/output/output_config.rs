use std::{
    collections::HashMap,
    fmt
};

use serde::{Serialize};
use arctk_attr::file;
use crate::{io::output::{OutputPlaneBuilder, OutputVolumeBuilder, PhotonCollectorBuilder}};


#[file]
#[derive(Serialize)]
pub struct OutputConfig {
    pub volumes: Option<HashMap<String, OutputVolumeBuilder>>,
    pub planes: Option<HashMap<String, OutputPlaneBuilder>>,
    pub photon_collectors: Option<HashMap<String, PhotonCollectorBuilder>>,
}

impl OutputConfig {
    pub fn n_volumes(&self) -> usize {
        match &self.volumes {
            Some(vol) => vol.iter().count(),
            None => 0,
        }
    }

    pub fn n_planes(&self) -> usize {
        match &self.planes {
            Some(plane) => plane.iter().count(),
            None => 0,
        }
    }

    pub fn n_photon_collectors(&self) -> usize {
        match &self.photon_collectors {
            Some(pc) => pc.iter().count(),
            None => 0,
        }
    }
}

impl fmt::Display for OutputConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OutputConfig {{\n")?;
        
        if let Some(volumes) = &self.volumes {
            write!(f, "  volumes: {{\n")?;
            for (name, volume) in volumes {
                write!(f, "    {}: {:?}\n", name, volume)?;
            }
            write!(f, "  }}\n")?;
        }
        
        if let Some(planes) = &self.planes {
            write!(f, "  planes: {{\n")?;
            for (name, plane) in planes {
                write!(f, "    {}: {:?}\n", name, plane)?;
            }
            write!(f, "  }}\n")?;
        }
        
        if let Some(photon_collectors) = &self.photon_collectors {
            write!(f, "  photon_collectors: {{\n")?;
            for (name, collector) in photon_collectors {
                write!(f, "    {}: {:?}\n", name, collector)?;
            }
            write!(f, "  }}\n")?;
        }
        
        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use json5;

    #[test]
    fn output_config_deserialise_test() {
        let conf_str = r#"
        {
            volumes: {
                full_vol: { boundary: [[0, 0, 0], [10, 10, 10]], res: [10, 10, 10], param: "energy" },
                partial_vol: { boundary: [[2.5, 2.5, 0], [2.5, 2.5, 10]], res: [100, 100, 10], param: "energy" },
            },
            planes: {
                bottom: { boundary: [[0, 0], [10, 10]], res: [10, 10], plane: "xy" },
            },
            photon_collectors: {
                terrain_collector: { kill_photons: false },
                sky_collector: { kill_photons: true },
            }
        }
        "#;
        
        // Deserialise from the provided string above. 
        let conf: OutputConfig = json5::from_str(conf_str).unwrap();
        
        // Check that all outputs make it through. 
        assert_eq!(conf.n_volumes(), 2);
        assert_eq!(conf.n_planes(), 1);
        assert_eq!(conf.n_photon_collectors(), 2);
    }

}