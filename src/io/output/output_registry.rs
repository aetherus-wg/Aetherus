use crate::{
    ord::Register,
    io::output::Output,
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
    
}