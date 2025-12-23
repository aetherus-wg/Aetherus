//! Attribute first-stage imager linker.

use crate::{
    err::Error,
    fmt_report,
    geom::{Orient, Ray},
    io::output::{AxisAlignedPlane, RasteriseBuilder, Rasteriser},
    math::{Dir3, Point3, Vec3},
    ord::{
        cartesian::{X, Y},
        Build, Link, Name, Set,
    },
    phys::{Material, Reflectance, ReflectanceBuilder},
    sim::attribute::Attribute,
    tools::{Binner, Range},
};
use arctk_attr::file;
use serde::{Deserialize, Deserializer};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum InterfaceFuture {
    Future((Name, Name)),      // Future
    Value(Material, Material), // Resolved ID
}

#[derive(Debug, Clone)]
pub enum IdFuture {
    Future(Name), // Future
    Value(usize), // Resolved ID
}

#[derive(Debug, Clone)]
pub enum PhotonCollectorFuture {
    Future((Name, bool)),
    Value(usize), // Resolved ID for out.regs.phot_col
}

#[derive(Debug, Clone)]
pub enum CcdFuture {
    Future((Name, [usize; 2], f64, Point3, Vec3, Binner)),
    Value(usize, f64, Orient, Binner), // Resolved ID and properties
}

#[derive(Debug, Clone)]
pub enum ImagerFuture {
    Future((Name, [usize; 2], f64, Point3, Vec3)),
    Value(usize, f64, Orient), // Resolved ID and properties
}

#[derive(Debug, Clone)]
pub enum SpectrometerFuture {
    Future((Name, [f64; 2], usize)),
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
            $ftype::Value(..) => Err(format!(
                "Attempted to unwrap already linked {}",
                stringify!($ftype)
            )),
        }
    };
}

type InterfaceConfig = (Name, Name);
impl<'de> Deserialize<'de> for InterfaceFuture {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let builder = InterfaceConfig::deserialize(deserializer)?;
        Ok(InterfaceFuture::Future(builder))
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

type PhotonCollectorConfig = (Name, bool);
impl<'de> Deserialize<'de> for PhotonCollectorFuture {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let builder = PhotonCollectorConfig::deserialize(deserializer)?;
        Ok(PhotonCollectorFuture::Future(builder))
    }
}

type CcdConfig = (Name, [usize; 2], f64, Point3, Vec3, Binner);
impl<'de> Deserialize<'de> for CcdFuture {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let builder = CcdConfig::deserialize(deserializer)?;
        Ok(CcdFuture::Future(builder))
    }
}

type ImagerConfig = (Name, [usize; 2], f64, Point3, Vec3);
impl<'de> Deserialize<'de> for ImagerFuture {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let builder = ImagerConfig::deserialize(deserializer)?;
        Ok(ImagerFuture::Future(builder))
    }
}

type SpectrometerConfig = (Name, [f64; 2], usize);
impl<'de> Deserialize<'de> for SpectrometerFuture {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let builder = SpectrometerConfig::deserialize(deserializer)?;
        Ok(SpectrometerFuture::Future(builder))
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
    /// Partially reflective mirror, reflection fraction.
    Mirror(f64),
    /// Spectrometer id, range, resolution.
    Spectrometer(SpectrometerFuture),
    /// Imager id, resolution, horizontal width (m), center, forward direction.
    Imager(ImagerFuture),
    /// Imager id, resolution, horizontal width (m), center, forward direction, wavelength binner (m).
    Ccd(CcdFuture),
    /// A purely reflecting material, with a provided reflectance model.
    /// The first coefficient is diffuse albedo, the second is specular.
    Reflector(ReflectorFuture),
    /// A photon collector, which collects the photon that interact with the linked entities.
    /// These photons can be optionally killed, or left to keep propogating.
    PhotonCollector(PhotonCollectorFuture),
    /// A chain of attributes where are executed in order.
    AttributeChain(Vec<AttributeFuture>),
    /// An output into the output plane object. This rasterises the photon packet into plane.
    Rasterise(IdFuture, RasteriseFuture),
    /// Hyperspectral output - output into a volume output
    Hyperspectral(IdFuture, AxisAlignedPlane),
}

impl<'a> Link<'a, usize> for AttributeFuture {
    type Inst = Self;
    #[inline]
    fn requires(&self) -> Vec<Name> {
        vec![]
    }

    fn link(mut self, reg: &'a Set<usize>) -> Result<Self, Error> {
        Ok(match self {
            Self::Interface(_) | Self::Mirror(_) => self,
            Self::Spectrometer(ref mut spec_future) => {
                if let SpectrometerFuture::Future((name, _range, _resolution)) = spec_future {
                    if let Some(id) = reg.get(&name) {
                        *spec_future = SpectrometerFuture::Value(*id);
                    }
                }
                self
            }
            Self::Imager(ref mut img_future) => {
                if let ImagerFuture::Future((name, _resolution, width, center, forward)) =
                    img_future
                {
                    if let Some(id) = reg.get(&name) {
                        let orient = Orient::new(Ray::new(*center, Dir3::from(*forward)));
                        *img_future = ImagerFuture::Value(*id, *width, orient)
                    }
                }
                self
            }
            Self::Ccd(ref mut ccd_future) => {
                if let CcdFuture::Future((name, _resolution, width, center, forward, binner)) =
                    ccd_future
                {
                    if let Some(id) = reg.get(&name) {
                        let orient = Orient::new(Ray::new(*center, Dir3::from(*forward)));
                        *ccd_future = CcdFuture::Value(*id, *width, orient, binner.clone())
                    }
                }
                self
            }
            Self::Reflector(ref mut ref_future) => {
                if let ReflectorFuture::Future(builder) = ref_future {
                    let ref_model = builder.build()?;
                    *ref_future = ReflectorFuture::Value(ref_model)
                }
                self
            }
            Self::PhotonCollector(ref mut pc_future) => {
                // TODO: `kill_photons` in attributes is not used but the output configuration
                // is used instead => Remove dead feature
                if let PhotonCollectorFuture::Future((name, _kill_photons)) = &pc_future {
                    if let Some(id) = reg.get(&name) {
                        return Ok(Self::PhotonCollector(PhotonCollectorFuture::Value(*id)));
                    }
                }
                self
            }
            Self::AttributeChain(attrs) => {
                let linked_attrs: Result<Vec<_>, _> =
                    attrs.iter().map(|a| a.clone().link(&reg)).collect();
                Self::AttributeChain(linked_attrs?)
            }
            Self::Rasterise(ref mut id_future, ref mut rasterise_future) => {
                if let RasteriseFuture::Future(builder) = rasterise_future {
                    let rasteriser = builder.build();
                    *rasterise_future = RasteriseFuture::Value(rasteriser);
                }
                if let IdFuture::Future(name) = id_future {
                    if let Some(id) = reg.get(&name) {
                        *id_future = IdFuture::Value(*id);
                    }
                }
                self
            }
            Self::Hyperspectral(ref mut id_future, ref _plane) => {
                if let IdFuture::Future(name) = id_future {
                    if let Some(id) = reg.get(&name) {
                        *id_future = IdFuture::Value(*id);
                    }
                }
                self
            }
        })
    }
}

impl<'a> Link<'a, Material> for AttributeFuture {
    type Inst = Self;

    #[inline]
    fn requires(&self) -> Vec<Name> {
        vec![]
    }

    fn link(mut self, mats: &'a Set<Material>) -> Result<Self::Inst, Error> {
        Ok(match self {
            Self::Interface(ref mut intf_future) => {
                if let InterfaceFuture::Future((in_name, out_name)) = intf_future {
                    let inside = mats.get(&in_name).ok_or(Error::Text(format!(
                        "Failed to link attribute-interface key: {}",
                        in_name
                    )))?;
                    let outside = mats.get(&out_name).ok_or(Error::Text(format!(
                        "Failed to link attribute-interface key: {}",
                        out_name
                    )))?;
                    *intf_future = InterfaceFuture::Value(inside.clone(), outside.clone());
                }
                self
            }
            _ => self,
        })
    }
}

impl Build for AttributeFuture {
    type Inst = Attribute;

    fn build(self) -> Result<Self::Inst, Error> {
        Ok(match self {
            Self::Interface(InterfaceFuture::Value(in_mat, out_mat)) => {
                Self::Inst::Interface(in_mat, out_mat)
            }
            Self::Mirror(abs) => Self::Inst::Mirror(abs),
            Self::Spectrometer(SpectrometerFuture::Value(id)) => Self::Inst::Spectrometer(id),
            Self::Imager(ImagerFuture::Value(id, width, orient)) => {
                Self::Inst::Imager(id, width, orient)
            }
            Self::Ccd(CcdFuture::Value(id, width, orient, binner)) => {
                Self::Inst::Ccd(id, width, orient, binner)
            }
            Self::Reflector(ReflectorFuture::Value(reflectance)) => {
                Self::Inst::Reflector(reflectance)
            }
            Self::Reflector(ReflectorFuture::Future(builder)) => {
                let ref_model = builder.build()?;
                Self::Inst::Reflector(ref_model)
            }
            Self::PhotonCollector(PhotonCollectorFuture::Value(id)) => {
                Self::Inst::PhotonCollector(id)
            }
            Self::Rasterise(IdFuture::Value(id), RasteriseFuture::Value(rasteriser)) => {
                Self::Inst::Rasterise(id, rasteriser)
            }
            Self::Hyperspectral(IdFuture::Value(id), plane) => Self::Inst::Hyperspectral(id, plane),
            Self::AttributeChain(attrs) => {
                let linked_attrs: Vec<_> = attrs
                    .iter()
                    .map(|a| a.clone().build())
                    .collect::<Result<Vec<_>, Error>>()?;

                Self::Inst::AttributeChain(linked_attrs)
            }
            _ => panic!(
                "Attempted to convert unlinked AttributeFuture: {} into Attribute",
                self
            ),
        })
    }
}

impl Display for AttributeFuture {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::Interface(intf_future) => {
                let (in_name, out_name) = unwrap_future!(InterfaceFuture, intf_future).expect(
                    "The attributes has already been built before displaying configuration",
                );
                write!(fmt, "Interface: {} :| {}", in_name, out_name)
            }
            Self::Mirror(abs) => {
                write!(fmt, "Mirror: {}% abs", abs * 100.0)
            }
            Self::Spectrometer(spec_future) => {
                let (id, [min, max], bins) = unwrap_future!(SpectrometerFuture, spec_future)
                    .expect(
                        "The attributes has already been built before displaying configuration",
                    );
                write!(
                    fmt,
                    "Spectrometer: {} {} ({})",
                    id,
                    Range::new(*min, *max),
                    bins
                )
            }
            Self::Imager(imager_future) => {
                let (id, res, width, center, forward) = unwrap_future!(ImagerFuture, imager_future)
                    .expect(
                        "The attributes has already been built before displaying configuration",
                    );
                writeln!(fmt, "Imager: ...")?;
                fmt_report!(fmt, id, "name");
                fmt_report!(fmt, &format!("[{} x {}]", res[X], res[Y]), "resolution");
                fmt_report!(fmt, width, "width (m)");
                fmt_report!(fmt, center, "center (m)");
                fmt_report!(fmt, forward, "forward");
                Ok(())
            }
            Self::Ccd(ccd_future) => {
                let (id, res, width, center, forward, binner) = unwrap_future!(
                    CcdFuture, ccd_future
                )
                .expect("The attributes has already been built before displaying configuration");
                writeln!(fmt, "Ccd: ...")?;
                fmt_report!(fmt, id, "name");
                fmt_report!(fmt, &format!("[{} x {}]", res[X], res[Y]), "resolution");
                fmt_report!(fmt, width, "width (m)");
                fmt_report!(fmt, center, "center (m)");
                fmt_report!(fmt, forward, "forward");
                fmt_report!(fmt, binner, "binner");
                Ok(())
            }
            Self::Reflector(ref_future) => {
                let ref_shim = unwrap_future!(ReflectorFuture, ref_future).expect(
                    "The attributes has already been built before displaying configuration",
                );
                writeln!(fmt, "Reflector: ...")?;
                fmt_report!(
                    fmt,
                    if let Some(diff_ref) = &ref_shim.diff_ref {
                        format!("{}", diff_ref)
                    } else {
                        String::from("none")
                    },
                    "diffuse reflectance"
                );
                fmt_report!(
                    fmt,
                    if let Some(spec_ref) = &ref_shim.spec_ref {
                        format!("{}", spec_ref)
                    } else {
                        String::from("none")
                    },
                    "specular reflectance"
                );
                fmt_report!(
                    fmt,
                    if let Some(specularity) = ref_shim.specularity {
                        format!("{}", specularity)
                    } else {
                        String::from("none")
                    },
                    "specularity"
                );
                Ok(())
            }
            Self::PhotonCollector(pc_future) => {
                let (id, kill_phot) = unwrap_future!(PhotonCollectorFuture, pc_future).expect(
                    "The attributes has already been built before displaying configuration",
                );
                writeln!(fmt, "Photon Collector: ...")?;
                fmt_report!(fmt, id, "name");
                fmt_report!(fmt, kill_phot, "kill photons?");
                Ok(())
            }
            Self::AttributeChain(ref attrs) => {
                writeln!(fmt, "Attribute Chain: ...")?;
                for attr in attrs {
                    attr.fmt(fmt)?;
                }
                Ok(())
            }
            Self::Rasterise(id_future, rast_future) => {
                let id = unwrap_future!(IdFuture, id_future).expect(
                    "The attributes has already been built before displaying configuration",
                );
                let rast_builder = unwrap_future!(RasteriseFuture, rast_future).expect(
                    "The attributes has already been built before displaying configuration",
                );
                writeln!(fmt, "Rasterise: ...")?;
                fmt_report!(fmt, id, "name");
                fmt_report!(fmt, rast_builder, "rasteriser");
                Ok(())
            }
            Self::Hyperspectral(id_future, ref plane) => {
                let id = unwrap_future!(IdFuture, id_future).expect(
                    "The attributes has already been built before displaying configuration",
                );
                writeln!(fmt, "Hyperspectral: ...")?;
                fmt_report!(fmt, id, "name");
                fmt_report!(fmt, plane, "plane");
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
            { PhotonCollector: ['pc', true]},
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
        let reg: Set<usize> = BTreeMap::new();
        let attr = AttributeFuture::Spectrometer(SpectrometerFuture::Value(1));
        let result = attr.link(&reg).unwrap();
        if let AttributeFuture::Spectrometer(SpectrometerFuture::Value(id)) = result {
            assert_eq!(id, 1);
        } else {
            panic!("Expected Spectrometer variant with Value");
        }
    }

    #[test]
    fn test_link_spectrometer_future_to_value() {
        let mut reg: Set<usize> = BTreeMap::new();
        reg.insert("name", 1);
        let attr =
            AttributeFuture::Spectrometer(SpectrometerFuture::Future(("name".to_string(), 0, 0)));
        let result = attr.link(&reg).unwrap();
        if let AttributeFuture::Spectrometer(SpectrometerFuture::Value(id)) = result {
            assert_eq!(id, 1);
        } else {
            panic!("Expected Spectrometer variant with Value");
        }
    }
}
