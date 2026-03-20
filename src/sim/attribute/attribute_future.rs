//! Attribute first-stage imager linker.

use crate::{
    err::Error,
    fmt_report,
    io::output::{RasteriseBuilder, Rasteriser},
    ord::{
        Build, Link, Name, Set,
    },
    phys::{Material, Reflectance, ReflectanceBuilder},
    sim::attribute::Attribute,
};
use arctk_attr::file;
use log::{error, warn};
use serde::{Deserialize, Deserializer};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum InterfaceFuture {
    Future(Name, Name),         // Future
    ImplicitFuture(Name),         // Future
    Implicit(Material),           // Resolved ID
    Value(Material, Material),    // Resolved ID
}

#[derive(Debug, Clone)]
pub enum IdFuture {
    Future(Name), // Future
    Value(usize), // Resolved ID
}

#[derive(Debug, Clone)]
pub enum ReflectorFuture {
    Future(ReflectanceBuilder),
    Value(Reflectance), // Resolved Reflectrance struct
}

macro_rules! unwrap_future {
    ($ftype:tt, $e:expr) => {
        match $e {
            $ftype::Future(future_data) => Ok(future_data),
            _ => Err(format!(
                "Attempted to unwrap already linked {}",
                stringify!($ftype)
            )),
        }
    };
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum InterfaceConfig {
    Explicit(Name, Name),
    Implicit(Name),
}
impl<'de> Deserialize<'de> for InterfaceFuture {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let builder = InterfaceConfig::deserialize(deserializer)?;
        Ok(
            match builder {
                InterfaceConfig::Explicit(inside, outside) => InterfaceFuture::Future(inside, outside),
                InterfaceConfig::Implicit(outside) => InterfaceFuture::ImplicitFuture(outside),
            }
        )
    }
}

impl<'de> Deserialize<'de> for IdFuture {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let builder = Name::deserialize(deserializer)?;
        Ok(IdFuture::Future(builder))
    }
}

impl<'de> Deserialize<'de> for ReflectorFuture {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let builder = ReflectanceBuilder::deserialize(deserializer)?;
        Ok(ReflectorFuture::Future(builder))
    }
}

#[derive(Debug, Clone)]
pub enum RasteriseFuture {
    Future(RasteriseBuilder),
    Value(Rasteriser),
}

impl<'de> Deserialize<'de> for RasteriseFuture {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let builder = RasteriseBuilder::deserialize(deserializer)?;
        Ok(RasteriseFuture::Future(builder))
    }
}

/// Surface attribute setup.
/// Handles detector linking.
#[file]
#[derive(Clone)]
pub enum AttributeFuture {
    /// Material interface, inside material name, outside material name.
    Interface(InterfaceFuture),
    /// A purely reflecting material, with a provided reflectance model.
    /// The first coefficient is diffuse albedo, the second is specular.
    Reflector(ReflectorFuture),
    /// Partially reflective mirror, reflection fraction.
    Mirror(f64),
    /// A photon collector, which collects the photon that interact with the linked entities.
    /// These photons can be optionally killed, or left to keep propogating.
    Detector(IdFuture),
    /// A chain of attributes where are executed in order.
    AttributeChain(Vec<AttributeFuture>),
}

impl AttributeFuture {
    pub fn resolve_material(self, in_mat: Option<Material>) -> Result<AttributeFuture, Error> {
        match &self {
            AttributeFuture::Interface(interf_fut) => {
                match interf_fut {
                    InterfaceFuture::Future(_, out_name) => {
                        return Err(Error::Linking(format!(
                            "Failed to resolve material for interface future with outside name: {}. Expected an implicit future.",
                            out_name.as_string()
                        )));
                    },
                    InterfaceFuture::ImplicitFuture(out_name) => {
                        return Err(Error::Linking(format!(
                            "Failed to resolve material for interface future with outside name: {}. Expected an implicit future.",
                            out_name.as_string()
                        )));
                    },
                    InterfaceFuture::Implicit(out_mat) => {
                        if let Some(in_mat) = in_mat {
                            Ok(AttributeFuture::Interface(InterfaceFuture::Value(in_mat, out_mat.clone())))
                        } else {
                            return Err(Error::Linking(format!(
                                "Failed to resolve material for implicit interface. Expected an inside material, but none provided."
                            )));
                        }
                    }
                    InterfaceFuture::Value(..) => {
                        warn!("Attempted to resolve material for already resolved interface future. Ignoring.");
                        Ok(self)
                    },
                }
            },
            _ => Ok(self),
        }
    }
}

impl<'a> Link<'a, usize> for AttributeFuture {
    type Inst = Self;
    fn requires(&self) -> Vec<Name> {
        vec![]
    }

    fn link(self, reg: &'a Set<usize>) -> Result<Self, Error> {
        Ok(match self {
            // Passthrough not covered or already linked values
            Self::Interface(_) | Self::Mirror(_) => self,
            Self::Reflector(ReflectorFuture::Value(_)) => self,
            // Link/Build futures with id
            // WARN: Is this a suitable place to build Reflector?
            Self::Reflector(ReflectorFuture::Future(ref_builder)) => {
                let ref_model = ref_builder.build(Name::new(""))?;
                Self::Reflector(ReflectorFuture::Value(ref_model))
            }
            Self::Detector(IdFuture::Value(_)) => self,
            Self::Detector(IdFuture::Future(name)) => {
                let id_future =
                if let Some(id) = reg.get(&name) {
                    IdFuture::Value(*id)
                } else {
                    IdFuture::Future(name)
                };
                Self::Detector(id_future)
            }
            Self::AttributeChain(attrs) => {
                let linked_attrs: Result<Vec<_>, _> =
                    attrs.iter().map(|a| a.clone().link(reg)).collect();
                Self::AttributeChain(linked_attrs?)
            }
        })
    }
}

impl Link<'_, Material> for AttributeFuture {
    type Inst = Self;

    fn requires(&self) -> Vec<Name> {
        vec![]
    }

    // TODO: Replace with linking to Arc<Material>
    fn link(mut self, mats: &Set<Material>) -> Result<Self::Inst, Error> {
        Ok(match self {
            Self::Interface(ref mut intf_future) => {
                match intf_future {
                    InterfaceFuture::Future(in_name, out_name) => {
                        let outside = mats.get(out_name).ok_or(Error::Text(
                            format!("Failed to link attribute-interface outside key: {out_name}")
                        ))?;
                        if let Some(inside) = mats.get(in_name) {
                            *intf_future = InterfaceFuture::Value(inside.clone(), outside.clone());
                        } else {
                            warn!("Failed to link attribute-interface inside key: {in_name}, fallback to Implicit definition");
                            *intf_future = InterfaceFuture::Implicit(outside.clone());
                        }
                    },
                    InterfaceFuture::ImplicitFuture(out_name) => {
                        let outside = mats.get(out_name).ok_or(Error::Text(
                            format!("Failed to link outside material {out_name} for implicit interface")
                        ))?;
                        *intf_future = InterfaceFuture::Implicit(outside.clone());
                    },
                    // WARN: Can't link outside and implicit inside in one go, need to call link
                    // again
                    // TODO: Find a way to link inside without Set<Material> in order to ensure
                    // "inside" is given
                    InterfaceFuture::Implicit(out_material) => {
                        let inside = mats.get(&Name::new("inside")).ok_or(Error::Text(
                            format!("Failed to link implicit material from \"inside\" entry")
                        ))?;
                        *intf_future = InterfaceFuture::Value(inside.clone(), out_material.clone());
                    }
                    // Already linked, do nothing.
                    InterfaceFuture::Value(_, _) => {
                    }
                };
                self
            }
            _ => self,
        })
    }
}

impl Build for AttributeFuture {
    type Inst = Attribute;
    type MetaInfo = Name;

    fn build(self, id: Self::MetaInfo) -> Result<Self::Inst, Error> {
        Ok(match self {
            Self::Interface(InterfaceFuture::Value(in_mat, out_mat)) => {
                Self::Inst::Interface(in_mat, out_mat)
            }
            Self::Reflector(ReflectorFuture::Value(reflectance)) => {
                Self::Inst::Reflector(reflectance)
            }
            Self::Reflector(ReflectorFuture::Future(ref_builder)) => {
                let ref_model = ref_builder.build(id)?;
                Self::Inst::Reflector(ref_model)
            }
            Self::Mirror(abs) => Self::Inst::Mirror(abs),
            Self::Detector(IdFuture::Value(id)) => {
                Self::Inst::Detector(id)
            }

            Self::AttributeChain(attrs) => {
                let linked_attrs: Vec<_> = attrs
                    .iter()
                    .map(|a| a.clone().build(id.clone()))
                    .collect::<Result<Vec<_>, Error>>()?;

                Self::Inst::AttributeChain(linked_attrs)
            }
            _ => Err(Error::Linking(format!(
                "Attempted to convert unlinked AttributeFuture: {:?} into Attribute",
                self)))?
        })
    }
}

impl Display for AttributeFuture {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::Interface(intf_future) => {
                match intf_future {
                    InterfaceFuture::Future(in_name, out_name) => {
                        write!(fmt, "Interface Future: {in_name} :| {out_name}")
                    },
                    InterfaceFuture::ImplicitFuture(out_name) => {
                        write!(fmt, "Interface Implicit Future: ... :| {out_name}")
                    },
                    _ => {
                        error!("Attempted to display already linked InterfaceFuture. This should have been converted to an Attribute by now.");
                        Err(std::fmt::Error)
                    },
                }
            }
            Self::Mirror(abs) => {
                write!(fmt, "Mirror: {}% abs", abs * 100.0)
            }
            Self::Reflector(ref_future) => {
                let ref_shim = unwrap_future!(ReflectorFuture, ref_future).expect(
                    "The attributes has already been built before displaying configuration",
                );
                writeln!(fmt, "Reflector: ...")?;
                fmt_report!(
                    fmt,
                    if let Some(diff_ref) = &ref_shim.diff_ref {
                        format!("{diff_ref}")
                    } else {
                        String::from("none")
                    },
                    "diffuse reflectance"
                );
                fmt_report!(
                    fmt,
                    if let Some(spec_ref) = &ref_shim.spec_ref {
                        format!("{spec_ref}")
                    } else {
                        String::from("none")
                    },
                    "specular reflectance"
                );
                fmt_report!(
                    fmt,
                    if let Some(specularity) = ref_shim.specularity {
                        format!("{specularity}")
                    } else {
                        String::from("none")
                    },
                    "specularity"
                );
                Ok(())
            }
            Self::Detector(id_future) => {
                let id = unwrap_future!(IdFuture, id_future)
                    .expect("The attributes has already been built before displaying configuration");
                writeln!(fmt, "Detector: ...")?;
                fmt_report!(fmt, id, "name");
                Ok(())
            }
            Self::AttributeChain(ref attrs) => {
                writeln!(fmt, "Attribute Chain: ...")?;
                for attr in attrs {
                    attr.fmt(fmt)?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use json5;
    use std::collections::BTreeMap;

    /// Checks that we can deserialise an attribute chain from a JSON 5 input.
    /// This is necessary for getting it to run through the linker chain.
    #[test]
    fn test_deserialise_attribute_chain() {
        let desr_str = r#"
        { AttributeChain: [
            { Detector: 'pc'},
            { Reflector: [null, {Tophat: [550e-9, 575e-9, 0.5]}, null]},
        ]}
        "#;

        let attr: AttributeFuture = json5::from_str(&desr_str).unwrap();
        match attr {
            AttributeFuture::AttributeChain(attrs) => {
                assert_eq!(attrs.iter().count(), 2);
            }
            _ => panic!("Unable to deserialise AttributeChain. "),
        }
    }

    #[test]
    fn test_link_spectrometer_value_unchanged() {
        let reg: Set<usize> = crate::ord::set::Set::new(BTreeMap::new());
        let attr = AttributeFuture::Detector(IdFuture::Value(1));
        let result = attr.link(&reg).unwrap();
        if let AttributeFuture::Detector(IdFuture::Value(id)) = result {
            assert_eq!(id, 1);
        } else {
            panic!("Expected Detector variant with Value");
        }
    }

    #[test]
    fn test_link_spectrometer_future_to_value() {
        let mut reg_map = BTreeMap::new();
        reg_map.insert(Name::new("name"), 1);
        let reg: Set<usize> = crate::ord::set::Set::new(reg_map);
        let attr =
            AttributeFuture::Detector(IdFuture::Future(Name::new("name")));
        let result = attr.link(&reg).unwrap();
        if let AttributeFuture::Detector(IdFuture::Value(id)) = result {
            assert_eq!(id, 1);
        } else {
            panic!("Expected Detector variant with Value");
        }
    }
}
