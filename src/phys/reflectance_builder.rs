use serde::{Serialize, Deserialize, Deserializer};
use serde::de::{SeqAccess, Visitor};

use crate::{
    err::Error,
    phys::SpectrumBuilder
};

use super::Reflectance;

#[derive(Serialize, Clone, Debug)]
pub struct ReflectanceBuilder {
    pub diff_ref: Option<SpectrumBuilder>,
    pub spec_ref: Option<SpectrumBuilder>,
    pub specularity: Option<f64>,
}

impl<'de> Deserialize<'de> for ReflectanceBuilder {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ReflectanceBuilderVisitor;
        impl<'de> Visitor<'de> for ReflectanceBuilderVisitor {
            type Value = ReflectanceBuilder;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an array with 3 elements: [diff_ref, spec_ref, specularity]")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let diff_ref: Option<SpectrumBuilder> = seq.next_element()?.unwrap_or(None);
                let spec_ref: Option<SpectrumBuilder> = seq.next_element()?.unwrap_or(None);
                let specularity: Option<f64> = seq.next_element()?.unwrap_or(None);

                Ok(ReflectanceBuilder {
                    diff_ref,
                    spec_ref,
                    specularity,
                })
            }
        }
        deserializer.deserialize_seq(ReflectanceBuilderVisitor)
    }


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
