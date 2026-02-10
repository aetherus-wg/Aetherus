use std::{collections::BTreeMap, fmt};

use crate::{
    data::HistogramBuilder,
    err::Error,
    fmt_report,
    geom::{Orient, Ray},
    img::ImageBuilder,
    io::output::{Detector, DetectorType, Output, OutputPlaneBuilder, OutputVolumeBuilder, PhotonCollector},
    math::{Dir3, Point3},
    ord::{Build, Name},
    tools::Binner
};
use arctk_attr::file;
use serde::Deserialize;

use super::{CcdBuilder, OutputRegistry};

#[derive(Debug, Clone, Deserialize)]
pub struct OrientBuilder {
    pos: Point3,
    forward: Dir3,
    up: Option<Dir3>,
    right: Option<Dir3>,
}

impl Build for OrientBuilder {
    type Inst = Orient;
    fn build(self) -> Result<Self::Inst, Error> {
        if self.up.is_none() && self.right.is_none() {
        }
        if let Some(up) = self.up {
            let _right = if let Some(right) = self.right {
                if up.dot(&right) == 0.0 {
                    return Err(Error::Build("Up and right vectors must be orthogonal for Orient".to_string()));
                }
                right
            } else {
                let right = up.cross(&self.forward);
                Dir3::new(right.x(), right.y(), right.z())
            };
            Ok(Orient::new_up(Ray::new(self.pos, self.forward), &up))
        } else if let Some(right) = self.right {
            let up = self.forward.cross(&right);
            let up_dir = Dir3::new(up.x(), up.y(), up.z());
            Ok(Orient::new_up(Ray::new(self.pos, self.forward), &up_dir))
        } else {
            return Err(Error::Build("At least one of \"up\" or \"right\" must be provided for Orient".to_string()));
        }
    }
}

#[file]
#[derive(Clone)]
pub struct OutputConfig {
    pub volumes: Option<BTreeMap<Name, OutputVolumeBuilder>>,
    pub planes: Option<BTreeMap<Name, OutputPlaneBuilder>>,
    pub photon_collectors: Option<BTreeMap<Name, PhotonCollector>>,
    pub spectra: Option<BTreeMap<Name, HistogramBuilder>>,
    pub images: Option<BTreeMap<Name, ImageBuilder>>,
    pub ccds: Option<BTreeMap<Name, CcdBuilder>>,
}

impl Build for OutputConfig {
    type Inst = Output;
    fn build(self) -> Result<Self::Inst, Error> {
        let reg = OutputRegistry::new_from_config(&self);
        // Volume output.
        let vol = match &self.volumes {
            Some(vols) => vols.values().map(|conf| conf.build()).collect(),
            None => vec![],
        };

        let plane = match &self.planes {
            Some(planes) => planes.values().map(|conf| conf.build()).collect(),
            None => vec![],
        };

        // Detectors
        // ---------

        let mut det_id = 0;
        let mut detectors: Vec<Detector> = vec![];

        let phot_cols = match &self.photon_collectors {
            Some(pcs) => {
                for (_key, _conf) in pcs.iter() {
                    detectors.push(Detector {
                        id: det_id,
                        det_type: DetectorType::PhotonCollector,
                    });
                    det_id += 1;
                }
                pcs.values().map(|conf| conf.clone()).collect()
            }
            None => vec![],
        };

        let specs = match &self.spectra {
            Some(specs) => {
                for (_key, _conf) in specs.iter() {
                    detectors.push(Detector {
                        id: det_id,
                        det_type: DetectorType::Spectrometer
                    });
                    det_id += 1;
                }
                specs.values()
                    .map(|conf| conf.build())
                    .collect()
            },
            None => vec![],
        };

        let images = match &self.images {
            Some(images) => {
                for (_key, conf) in images.iter() {
                    detectors.push(Detector {
                        id: det_id,
                        det_type: DetectorType::Imager {
                            width: conf.width(),
                            height: conf.height(),
                            orient: conf.orient().clone().build()?,
                        }
                    });
                    det_id += 1;
                }
                images.values()
                    .map(|conf| conf.build())
                    .collect()
            }
            None => vec![],
        };

        let ccds = match &self.ccds {
            Some(ccds) => {
                for (_key, conf) in ccds.iter() {
                    let orient = conf.orient().clone().build()?;
                    let binner = Binner::new(conf.range()?, conf.bins());
                    detectors.push( Detector {
                        id: det_id,
                        det_type: DetectorType::Ccd {
                            width: conf.width(),
                            height: conf.height(),
                            orient,
                            binner,
                        }
                    });
                    det_id += 1;
                }
                ccds.values()
                    .map(|conf| conf.build())
                    .collect()
            }
            None => vec![],
        };

        Ok(Output {
            vol,
            plane,
            detectors,
            phot_cols,
            specs,
            images,
            ccds,
            reg,
        })
    }
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
}

impl fmt::Display for OutputConfig {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), std::fmt::Error> {
        writeln!(fmt, "...")?;

        match &self.volumes {
            Some(vols) => {
                fmt_report!(fmt, "...", "volume outputs");
                for (key, vol) in vols {
                    fmt_report!(fmt, vol, key);
                }
            },
            None => fmt_report!(fmt, "none", "volume outputs"),
        }

        match &self.planes {
            Some(planes) => {
                fmt_report!(fmt, "...", "plane outputs");
                for (key, plane) in planes {
                    fmt_report!(fmt, plane, key);
                }
            },
            None => fmt_report!(fmt, "none", "plane outputs"),
        }

        match &self.photon_collectors {
            Some(pcs) => {
                fmt_report!(fmt, "...", "photon collectors");
                for (key, pc) in pcs {
                    fmt_report!(fmt, pc, key);
                }
            },
            None => fmt_report!(fmt, "none", "photon collectors"),
        }

        match &self.spectra {
            Some(spectra) => {
                fmt_report!(fmt, "...", "spectra");
                for (key, spec) in spectra {
                    fmt_report!(fmt, spec, key);
                }
            },
            None => fmt_report!(fmt, "none", "spectra"),
        }

        match &self.images {
            Some(imgs) => {
                fmt_report!(fmt, "...", "images");
                for (key, img) in imgs {
                    fmt_report!(fmt, img, key);
                }
            },
            None => fmt_report!(fmt, "none", "images"),
        }

        match &self.ccds {
            Some(ccds) => {
                fmt_report!(fmt, "...", "ccds");
                for (key, ccd) in ccds {
                    fmt_report!(fmt, ccd, key);
                }
            },
            None => fmt_report!(fmt, "none", "ccds"),
        }

        Ok(())
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
                small_image: {
                    res: [1024, 768],
                    width: 0.05,
                    height: 0.05,
                    orient: {
                        pos: [0.0, 0.0, 0.0],
                        forward: [0.0, 0.0, 1.0],
                        up: [0.0, 1.0, 0.0],
                    },
                },
                larger_image: {
                    res: [1920, 1080],
                    width: 0.05,
                    height: 0.05,
                    orient: {
                        pos: [0.0, 0.0, 0.0],
                        forward: [0.0, 0.0, 1.0],
                        up: [0.0, 1.0, 0.0],
                    },
                },
                uhd_image: {
                    res: [3840, 2160],
                    width: 0.05,
                    height: 0.05,
                    orient: {
                        pos: [0.0, 0.0, 0.0],
                        forward: [0.0, 0.0, 1.0],
                        up: [0.0, 1.0, 0.0],
                    },
                },
            },
            ccds: {
                default_ccd: {
                    res: [1024, 768],
                    range: [400E-9, 800E-9],
                    bins: 10,
                    width: 0.05,
                    height: 0.05,
                    orient: {
                        pos: [0.0, 0.0, 0.0],
                        forward: [0.0, 0.0, 1.0],
                        up: [0.0, 1.0, 0.0],
                    },
                },
            },
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
    }

    #[test]
    fn test_build_output() {
        let conf_str = r#"
        {
            volumes: {
                full_vol: { boundary: [[0, 0, 0], [10, 10, 10]], res: [100, 100, 100], param: "energy" },
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
                small_image: {
                    res: [1024, 768],
                    width: 0.05,
                    height: 0.05,
                    orient: {
                        pos: [0.0, 0.0, 0.0],
                        forward: [0.0, 0.0, 1.0],
                        up: [0.0, 1.0, 0.0],
                    },
                },
                larger_image: {
                    res: [1920, 1080],
                    width: 0.05,
                    height: 0.05,
                    orient: {
                        pos: [0.0, 0.0, 0.0],
                        forward: [0.0, 0.0, 1.0],
                        up: [0.0, 1.0, 0.0],
                    },
                },
                uhd_image: {
                    res: [3840, 2160],
                    width: 0.05,
                    height: 0.05,
                    orient: {
                        pos: [0.0, 0.0, 0.0],
                        forward: [0.0, 0.0, 1.0],
                        up: [0.0, 1.0, 0.0],
                    },
                },
            },
            ccds: {
                default_ccd: {
                    res: [1024, 768],
                    range: [400E-9, 800E-9],
                    bins: 10,
                    width: 0.05,
                    height: 0.05,
                    orient: {
                        pos: [0.0, 0.0, 0.0],
                        forward: [0.0, 0.0, 1.0],
                        up: [0.0, 1.0, 0.0],
                    },
                },
            },
        }
        "#;
        // Deserialise from the provided string above.
        let conf: OutputConfig = json5::from_str(conf_str).unwrap();
        let _out = conf.build();

        // TODO: Implement some tests for the output object here.
    }
}
