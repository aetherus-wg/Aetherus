use crate::{fs::Save, phys::Photon, err::Error, tools::ProgressBar, fmt_report};
use std::{path::Path, fmt::Display, ops::AddAssign, fs::File, io::Write, sync::{Arc, Mutex}};

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
        let mut file = File::create(path)?;
        
        // To reduce the time to run, I am manually do my won CSV write, directly from this vec. 
        let headings = vec!["pos_x", "pos_y", "pos_z", "dir_x", "dir_y", "dir_z", "wavelength", "weight"];
        write!(file, "{}", headings[0])?;
        for heading in headings.iter().skip(1) {
            write!(file, ",{}", heading)?;
        }
        writeln!(file)?;

        // This can potentually
        let mut pb = ProgressBar::new("Saving Photons", self.photons.iter().count());

        // Write the properties of each of the photons to the output table. 
        for phot in self.photons.iter() {
            writeln!(file, "{},{},{},{},{},{},{},{}",
                phot.ray().pos().x(),
                phot.ray().pos().y(),
                phot.ray().pos().z(),
                phot.ray().dir().x(),
                phot.ray().dir().y(),
                phot.ray().dir().z(),
                phot.wavelength(),
                phot.weight(),
            )?;
            pb.tick();
        }

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