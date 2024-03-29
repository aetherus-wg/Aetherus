//! Output data structure.

use crate::{
    access, clone,
    data::Histogram,
    err::Error,
    fmt_report,
    fs::Save,
    geom::Cube,
    img::Image,
    ord::{Register, X, Y, Z},
    util::fmt::DataCube,
};
use ndarray::Array3;
use std::{
    fmt::{Display, Formatter},
    ops::AddAssign,
    path::Path,
};

use super::PhotonCollector;

/// MCRT output data.
#[derive(Clone)]
pub struct Output<'a> {
    /// Measured volume.
    boundary: Cube,
    /// Cell volume [m^3].
    cell_vol: f64,

    /// Emission power.
    pub emission: Array3<f64>,
    /// Photo-energy.
    pub energy: Array3<f64>,
    /// Absorptions.
    pub absorptions: Array3<f64>,
    /// Wavelength shifts.
    pub shifts: Array3<f64>,
    /// Flux - the energy density travelling through each voxel.
    pub flux: Array3<f64>,

    /// Spectrometer name register.
    spec_reg: &'a Register,
    /// Imager name register.
    img_reg: &'a Register,
    /// CCD name register.
    ccd_reg: &'a Register,
    /// Photon collectors.
    phot_col_reg: &'a Register,
    /// Spectrometer data.
    pub specs: Vec<Histogram>,
    /// Image data.
    pub imgs: Vec<Image>,
    /// Ccd data.
    pub ccds: Vec<Array3<f64>>,

    /// Photo data.
    pub photos: Vec<Image>,
    /// Photon collectors.
    pub phot_cols: Vec<PhotonCollector>,
}

impl<'a> Output<'a> {
    access!(boundary: Cube);
    clone!(cell_vol: f64);
    access!(spec_reg: Register);
    access!(img_reg: Register);
    access!(ccd_reg: Register);

    /// Construct a new instance.
    #[allow(clippy::too_many_arguments)]
    #[inline]
    #[must_use]
    pub fn new(
        boundary: Cube,
        res: [usize; 3],
        spec_reg: &'a Register,
        img_reg: &'a Register,
        ccd_reg: &'a Register,
        phot_col_reg: &'a Register,
        specs: Vec<Histogram>,
        imgs: Vec<Image>,
        ccds: Vec<Array3<f64>>,
        photos: Vec<Image>,
        phot_cols: Vec<PhotonCollector>,
    ) -> Self {
        debug_assert!(res[X] > 0);
        debug_assert!(res[Y] > 0);
        debug_assert!(res[Z] > 0);

        let cell_vol = boundary.vol() / (res[X] * res[Y] * res[Z]) as f64;

        Self {
            boundary,
            cell_vol,
            emission: Array3::zeros(res),
            energy: Array3::zeros(res),
            absorptions: Array3::zeros(res),
            shifts: Array3::zeros(res),
            flux: Array3::zeros(res),
            spec_reg,
            img_reg,
            ccd_reg,
            phot_col_reg: phot_col_reg,
            specs,
            imgs,
            ccds,
            photos,
            phot_cols,
        }
    }
}

impl AddAssign<&Self> for Output<'_> {
    #[inline]
    fn add_assign(&mut self, rhs: &Self) {
        self.emission += &rhs.emission;
        self.energy += &rhs.energy;
        self.absorptions += &rhs.absorptions;
        self.shifts += &rhs.shifts;

        for (a, b) in self.specs.iter_mut().zip(&rhs.specs) {
            *a += b;
        }

        for (a, b) in self.imgs.iter_mut().zip(&rhs.imgs) {
            *a += b;
        }

        for (a, b) in self.ccds.iter_mut().zip(&rhs.ccds) {
            *a += b;
        }

        for (a, b) in self.photos.iter_mut().zip(&rhs.photos) {
            *a += b;
        }

        for (a, b) in self.phot_cols.iter_mut().zip(&rhs.phot_cols) {
            *a += b;
        }
    }
}

impl Save for Output<'_> {
    #[inline]
    fn save_data(&self, out_dir: &Path) -> Result<(), Error> {
        let path = out_dir.join("emission_density.nc");
        (&self.emission / self.cell_vol).save(&path)?;

        let path = out_dir.join("energy_density.nc");
        (&self.energy / self.cell_vol).save(&path)?;

        let path = out_dir.join("absorption_density.nc");
        (&self.absorptions / self.cell_vol).save(&path)?;

        let path = out_dir.join("shift_density.nc");
        (&self.shifts / self.cell_vol).save(&path)?;

        for (name, index) in self.spec_reg.set().map().iter() {
            self.specs[*index].save(&out_dir.join(&format!("spectrometer_{}.csv", name)))?;
        }

        for (name, index) in self.img_reg.set().map().iter() {
            self.imgs[*index].save(&out_dir.join(&format!("img_{}.png", name)))?;
        }

        for (name, index) in self.ccd_reg.set().map().iter() {
            self.ccds[*index].save(&out_dir.join(&format!("ccd_{}.nc", name)))?;
        }

        for (n, photo) in self.photos.iter().enumerate() {
            photo.save(&out_dir.join(&format!("photo_{:03}.png", n)))?;
        }

        for (name, index) in self.phot_col_reg.set().map().iter() {
            self.phot_cols[*index]
                .save(&out_dir.join(&format!("photon_collector_{}.csv", name)))?;
        }

        Ok(())
    }
}

impl Display for Output<'_> {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        writeln!(fmt, "...")?;
        fmt_report!(fmt, self.boundary, "boundary");
        fmt_report!(fmt, self.cell_vol, "cell volume (m^3)");

        fmt_report!(fmt, DataCube::new(&self.emission), "emission data");
        fmt_report!(fmt, DataCube::new(&self.energy), "energy data");
        fmt_report!(
            fmt,
            DataCube::new(&self.absorptions),
            "absorbed energy data"
        );
        fmt_report!(fmt, DataCube::new(&self.shifts), "shifted energy data");

        fmt_report!(fmt, self.spec_reg, "spectrometer register");
        fmt_report!(fmt, self.img_reg, "imager register");
        fmt_report!(fmt, self.ccd_reg, "ccd register");

        fmt_report!(fmt, self.specs.len(), "spectrometers");
        fmt_report!(fmt, self.imgs.len(), "images");
        fmt_report!(fmt, self.ccds.len(), "ccds");

        fmt_report!(fmt, self.photos.len(), "photos");
        fmt_report!(fmt, self.phot_cols.len(), "photon collectors");
        Ok(())
    }
}
