use std::collections::BTreeMap;

use crate::{
    io::output::OutputConfig, ord::{Name, Register},
};

#[derive(Clone)]
pub struct OutputRegistry {
    pub vol_reg: Register,
    pub plane_reg: Register,
    pub detectors_reg: Register,
    pub phot_cols_reg: Register,
    pub spec_reg: Register,
    pub images_reg: Register,
    pub ccd_reg: Register,
}

fn out_keys<T>(out: &Option<BTreeMap<Name, T>>) -> Vec<Name> {
    let keys = match &out {
        Some(inner) => inner.keys().map(|k| k.clone()).collect(),
        None => vec![]
    };
    keys
}

impl OutputRegistry {
    pub fn new_from_config(out: &OutputConfig) -> OutputRegistry {
        // Get the keys for the volume outputs.
        let vol_reg = Register::new(out_keys(&out.volumes));

        // Get the keys for the plane outputs.
        let plane_reg = Register::new(out_keys(&out.planes));

        // Get the keys for photon collector outputs.
        let phot_cols_reg = Register::new(out_keys(&out.photon_collectors));

        let spec_reg = Register::new(out_keys(&out.spectra));
        let images_reg = Register::new(out_keys(&out.images));
        let ccd_reg = Register::new(out_keys(&out.ccds));

        let detector_keys = vec![out_keys(&out.photon_collectors), out_keys(&out.spectra), out_keys(&out.images), out_keys(&out.ccds)]
            .into_iter()
            .flatten()
            .collect::<Vec<Name>>();
        let detectors_reg = Register::new(detector_keys);

        Self {
            vol_reg,
            plane_reg,
            detectors_reg,
            phot_cols_reg,
            spec_reg,
            images_reg,
            ccd_reg,
        }
    }
}
