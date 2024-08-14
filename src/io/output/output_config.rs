use std::{
    collections::BTreeMap,
    fmt
};

use serde::Serialize;
use arctk_attr::file;
use crate::{
    data::HistogramBuilder, 
    img::ImageBuilder, 
    io::output::{OutputPlaneBuilder, OutputVolumeBuilder, PhotonCollectorBuilder, Output},
    ord::Name};

use super::{CcdBuilder, OutputRegistry};


#[file]
#[derive(Serialize)]
pub struct OutputConfig {
    pub volumes: Option<BTreeMap<Name, OutputVolumeBuilder>>,
    pub planes: Option<BTreeMap<Name, OutputPlaneBuilder>>,
    pub photon_collectors: Option<BTreeMap<Name, PhotonCollectorBuilder>>,
    pub spectra: Option<BTreeMap<Name, HistogramBuilder>>,
    pub images: Option<BTreeMap<Name, ImageBuilder>>,
    pub ccds: Option<BTreeMap<Name, CcdBuilder>>,
    pub photos: Option<BTreeMap<Name, ImageBuilder>>,
}

impl OutputConfig {

    pub fn build(&self) -> Output {
        let reg = OutputRegistry::new_from_config(self);
        // Volume output. 
        let vol = match &self.volumes {
            Some(vols) => {
                vols.iter().map(|(_key, conf)| {
                    conf.build()
                })
                .collect()
            }, 
            None => vec![]
        };

        let plane = match &self.planes {
            Some(planes) => {
                planes.iter().map(|(_key, conf)| {
                    conf.build()
                })
                .collect()
            }, 
            None => vec![]
        };

        let phot_cols = match &self.photon_collectors {
            Some(pcs) => {
                pcs.iter().map(|(_key, conf)| {
                    conf.build()
                })
                .collect()
            }, 
            None => vec![]
        };

        let specs = match &self.spectra {
            Some(specs) => {
                specs.iter().map(|(_key, conf)| {
                    conf.build()
                })
                .collect()
            }, 
            None => vec![]
        };

        let imgs = match &self.images {
            Some(imgs) => {
                imgs.iter().map(|(_key, conf)| {
                    conf.build()
                })
                .collect()
            }, 
            None => vec![]
        };

        let ccds = match &self.ccds {
            Some(ccds) => {
                ccds.iter().map(|(_key, conf)| {
                    conf.build()
                })
                .collect()
            }, 
            None => vec![]
        };

        let photos = match &self.photos {
            Some(phots) => {
                phots.iter().map(|(_key, conf)| {
                    conf.build()
                })
                .collect()
            }, 
            None => vec![]
        };

        Output {
            vol,
            plane,
            phot_cols,
            specs,
            imgs,
            ccds,
            photos,
            reg,
        }
    }

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

    pub fn n_spectra(&self) -> usize {
        match &self.spectra {
            Some(spec) => spec.iter().count(),
            None => 0,
        }
    }

    pub fn n_images(&self) -> usize {
        match &self.images {
            Some(img) => img.iter().count(),
            None => 0,
        }
    }

    pub fn n_ccds(&self) -> usize {
        match &self.ccds {
            Some(ccd) => ccd.iter().count(),
            None => 0,
        }
    }

    pub fn n_photos(&self) -> usize {
        match &self.photos {
            Some(phot) => phot.iter().count(),
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

        // TODO: Add output for spectra. 
        // TODO: Add output for images. 
        // TODO: Add output for CCDs. 
        // TODO: Add output for photos. 
        
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
            },
            spectra: {
                spec: {min: 400E-9, max: 800E-9, bins: 500},
            },
            images: {
                small_image: { res: [1024, 768] },
                larger_image: { res: [1920, 1080] },
                uhd_image: { res: [3840, 2160] },
            },
            ccds: {
                default_ccd: { res: [1024, 768], bins: 10},
            },
            photos: {
                small_image: { res: [1024, 768] },
                larger_image: { res: [1920, 1080] },
                uhd_image: { res: [3840, 2160] },
            }
        }
        "#;
        
        // Deserialise from the provided string above. 
        let conf: OutputConfig = json5::from_str(conf_str).unwrap();
        
        // Check that all outputs make it through. 
        assert_eq!(conf.n_volumes(), 2);
        assert_eq!(conf.n_planes(), 1);
        assert_eq!(conf.n_photon_collectors(), 2);
        assert_eq!(conf.n_spectra(), 1);
        assert_eq!(conf.n_images(), 3);
        assert_eq!(conf.n_ccds(), 1);
        assert_eq!(conf.n_photos(), 3);
    }

    #[test]
    fn test_build_output() {
        let conf_str = r#"
        {
            volumes: {
                full_vol: { boundary: [[0, 0, 0], [10, 10, 10]], res: [8000, 8000, 1000], param: "energy" },
                partial_vol: { boundary: [[2.5, 2.5, 0], [2.5, 2.5, 10]], res: [100, 100, 10], param: "energy" },
            },
            planes: {
                bottom: { boundary: [[0, 0], [10, 10]], res: [10, 10], plane: "xy" },
            },
            photon_collectors: {
                terrain_collector: { kill_photons: false },
                sky_collector: { kill_photons: true },
            },
            spectra: {
                spec: {min: 400E-9, max: 800E-9, bins: 500},
            },
            images: {
                small_image: { res: [1024, 768] },
                larger_image: { res: [1920, 1080] },
                uhd_image: { res: [3840, 2160] },
            },
            ccds: {
                default_ccd: { res: [1024, 768], bins: 10},
            },
            photos: {
                small_image: { res: [1024, 768] },
                larger_image: { res: [1920, 1080] },
                uhd_image: { res: [3840, 2160] },
            }
        }
        "#;
        // Deserialise from the provided string above. 
        let conf: OutputConfig = json5::from_str(conf_str).unwrap();
        let _out = conf.build();

        // TODO: Implement some tests for the output object here. 
    }
}