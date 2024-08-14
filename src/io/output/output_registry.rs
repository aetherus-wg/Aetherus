use crate::{
    io::output::OutputConfig, ord::{Name, Register}
};

pub struct OutputRegistry {
    pub vol_reg: Register,
    pub plane_reg: Register,
    pub phot_cols_reg: Register,
    pub spec_reg: Register,
    pub img_reg: Register, 
    pub ccd_reg: Register,
}

impl OutputRegistry {
    pub fn new_from_config(out: &OutputConfig) -> OutputRegistry {
        // Get the keys for the volume outputs. 
        let vol_keys = match &out.volumes {
            Some(vols) => vols.keys().map(|k| k.clone()).collect(),
            None => vec![]
        };
        let vol_reg = Register::new(vol_keys);

        // Get the keys for the plane outputs. 
        let plane_keys = match &out.planes {
            Some(planes) => planes.keys().map(|k| k.clone()).collect(),
            None => vec![]
        };
        let plane_reg = Register::new(plane_keys);

        // Get the keys for photon collector outputs. 
        let photcol_keys = match &out.photon_collectors {
            Some(photcol) => photcol.keys().map(|k| k.clone()).collect(),
            None => vec![]
        };
        let phot_cols_reg = Register::new(photcol_keys);

        let spec_reg = Register::new(vec![]);
        let img_reg = Register::new(vec![]);
        let ccd_reg = Register::new(vec![]);

        Self {
            vol_reg,
            plane_reg,
            phot_cols_reg,
            spec_reg,
            img_reg,
            ccd_reg,
        }
    }
}