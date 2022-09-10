use crate::{fs::Save, phys::Photon, err::Error, data::Table, fmt_report};
use std::{path::Path, fmt::Display, ops::AddAssign};

#[derive(Default, Clone)]
pub struct PhotonCollector {
    /// The vector of collected photons. 
    pub photons: Vec<Photon>,
    /// Whether the collector should kill the photon when it has been collected. 
    pub kill_photon: bool,
}

impl PhotonCollector {
    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    pub fn collect_photon(&mut self, phot: &mut Photon) {
        self.photons.push(phot.clone());

        if self.kill_photon {
            phot.kill();
        }
    }
}

impl Save for PhotonCollector {
    /// Loads the fields of the photon into a vec of vecs and outputs using a table to CSV. 
    fn save_data(&self, path: &Path) -> Result<(), Error> {  
        let mut rows = Vec::with_capacity(self.photons.iter().count());      
        for phot in self.photons.iter() {
            rows.push(vec![
                phot.ray().pos().x(),
                phot.ray().pos().y(),
                phot.ray().pos().z(),
                phot.ray().dir().x(),
                phot.ray().dir().y(),
                phot.ray().dir().z(),
                phot.wavelength(),
            ]);
        }

        let tab = Table::<f64>::new(
            vec!["pos_x".to_string(), "pos_y".to_string(), "pos_z".to_string(), "dir_x".to_string(), "dir_y".to_string(), "dir_z".to_string(), "lambda".to_string()],
            rows
        );
        tab.save_data(path)?;

        Ok(())
    }
}

impl AddAssign<&Self> for PhotonCollector {
    #[inline]
    fn add_assign(&mut self, rhs: &Self) {
        self.photons.extend(rhs.photons.clone());
    }
}

impl Display for PhotonCollector {
    #[inline]
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(fmt, "PhotonCollector: ")?;
        fmt_report!(fmt, self.kill_photon, "kill on collect");
        Ok(())
    }
}