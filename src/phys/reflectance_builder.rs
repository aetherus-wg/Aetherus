use crate::{
    err::Error,
    phys::SpectrumBuilder
};

use super::Reflectance;

pub type ReflectanceBuilderShim = (
    Option<SpectrumBuilder>, 
    Option<SpectrumBuilder>, 
    Option<f64>,
);

pub struct ReflectanceBuilder {
    pub diff_ref: Option<SpectrumBuilder>,
    pub spec_ref: Option<SpectrumBuilder>,
    pub specularity: Option<f64>,
}

impl ReflectanceBuilder {
    pub fn build(&self) -> Result<Reflectance, Error> {
        let ref_model = if self.diff_ref.is_some() {
            if self.spec_ref.is_some() {
                // Check that the specularity of the reflector is defined.
                assert!(self.specularity.is_some());
                Reflectance::Composite {
                    diffuse_refspec: self.diff_ref.clone().unwrap().build()?,
                    specular_refspec: self.spec_ref.clone().unwrap().build()?,
                    specularity: self.specularity.unwrap(),
                }
            } else {
                Reflectance::Lambertian {
                    refspec: self.diff_ref.clone().unwrap().build()?,
                }
            }
        } else {
            Reflectance::Specular {
                refspec: self.spec_ref.clone().unwrap().build()?,
            }
        };

        Ok(ref_model)
    }
}

impl From<ReflectanceBuilderShim> for ReflectanceBuilder {
    fn from(value: ReflectanceBuilderShim) -> Self {
        ReflectanceBuilder {
            diff_ref: value.0,
            spec_ref: value.1,
            specularity: value.2,
        }
    }
}