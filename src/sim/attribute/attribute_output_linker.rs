
use crate::{
    err::Error, fmt_report, 
    io::output::{AxisAlignedPlane, OutputRegistry, RasteriseBuilder}, 
    math::{Point3, Vec3}, ord::{cartesian::{X, Y}, Name}, 
    phys::{ReflectanceBuilderShim, ReflectanceBuilder}, 
    tools::Binner,
    sim::AttributeMaterialLinker,
    geom::{Orient, Ray},
    math::Dir3,
};
use arctk_attr::file;

use std::fmt::{Display, Formatter};

#[file]
pub enum AttributeOutputLinker {
    /// Material interface, inside material name, outside material name.
    Interface(Name, Name),
    /// Partially reflective mirror, reflection fraction.
    Mirror(f64),
    /// Spectrometer id, range, resolution.
    Spectrometer(Name, [f64; 2], usize),
    /// Imager id, resolution, horizontal width (m), center, forward direction.
    Imager(Name, [usize; 2], f64, Point3, Vec3),
    /// Imager id, resolution, horizontal width (m), center, forward direction, wavelength binner (m).
    Ccd(Name, [usize; 2], f64, Point3, Vec3, Binner),
    /// A purely reflecting material, with a provided reflectance model.
    /// The first coefficient is diffuse albedo, the second is specular.
    Reflector(ReflectanceBuilderShim),
    /// A photon collector, which collects the photon that interact with the linked entities.
    /// These photons can be optionally killed, or left to keep propogating.
    PhotonCollector(Name, bool),
    /// A chain of attributes where are executed in order. 
    AttributeChain(Vec<AttributeOutputLinker>),
    /// An output into the output plane object. This rasterises the photon packet into plane. 
    Rasterise(Name, RasteriseBuilder),
    /// Hyperspectral output - output into a volume output
    Hyperspectral(Name, AxisAlignedPlane),
}

impl AttributeOutputLinker {
    fn link(&self, reg: &OutputRegistry) -> Result<AttributeMaterialLinker, Error> {
        Ok(match self {
            Self::Interface(inside, outside) => AttributeMaterialLinker::Interface(inside.clone(), outside.clone()),
            Self::Mirror(r) => AttributeMaterialLinker::Mirror(*r),
            Self::Spectrometer(id, ..) => AttributeMaterialLinker::Spectrometer(
                *reg.spec_reg.set().get(&id)
                    .unwrap_or_else(|| panic!("Failed to link attribute-spectrometer key: {}", id)),
            ),
            Self::Imager(id, _resolution, width, center, forward) => AttributeMaterialLinker::Imager(
                *reg.img_reg.set().get(&id)
                    .unwrap_or_else(|| panic!("Failed to link attribute-imager key: {}", id)),
                *width,
                Orient::new(Ray::new(*center, Dir3::from(*forward))),
            ),
            Self::Ccd(id, _resolution, width, center, forward, binner) => AttributeMaterialLinker::Ccd(
                *reg.ccd_reg.set().get(&id)
                    .unwrap_or_else(|| panic!("Failed to link attribute-ccd key: {}", id)),
                *width,
                Orient::new(Ray::new(*center, Dir3::from(*forward))),
                binner.clone(),
            ),
            Self::Reflector(ref_shim) => {
                let ref_build: ReflectanceBuilder = ref_shim.clone().into();
                let ref_model = ref_build.build()?;
                AttributeMaterialLinker::Reflector(ref_model)
            }
            Self::PhotonCollector(ref id, _kill_photons) => {
                AttributeMaterialLinker::PhotonCollector(*reg.phot_cols_reg.set().get(&id).unwrap_or_else(|| {
                    panic!("Failed to link attribute-photon collector key : {}", id)
                }))
            },
            Self::AttributeChain(attrs) => {
                let linked_attrs: Result<Vec<_>, _> = attrs.iter()
                    .map(|a| a.link(reg))
                    .collect();

                AttributeMaterialLinker::AttributeChain(linked_attrs?)
            }
            Self::Rasterise(ref id, ref rast_build) => {
                let linked_id = *reg.plane_reg.set().get(&id)
                    .unwrap_or_else(|| panic!("Failed to link attribute-rasterise key: {}", id));
                AttributeMaterialLinker::Rasterise(linked_id, rast_build.build())
            },
            Self::Hyperspectral(name, plane) => {
                let id = reg.vol_reg.set().get(&name)
                    .expect(format!("Failed to like attribute-volume key: {}", name).as_str());
                AttributeMaterialLinker::Hyperspectral(*id, plane.clone())
            }
        })
    }
}