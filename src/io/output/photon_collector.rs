use crate::{err::Error, fmt_report, fs::Save, phys::Photon, tools::ProgressBar};
use std::{fmt::Display, fs::File, io::Write, ops::AddAssign, path::Path};

#[derive(Default, Clone, Debug)]
pub struct PhotonCollector {
    /// The vector of collected photons.
    pub photons: Vec<Photon>,
    /// Whether the collector should kill the photon when it has been collected.
    pub kill_photon: bool,
}

impl PhotonCollector {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn collect_photon(&mut self, phot: &mut Photon) {
        self.photons.push(phot.clone());

        if self.kill_photon {
            phot.kill();
        }
    }

    pub fn nphoton(&self) -> usize {
        self.photons.iter().count()
    }
}

impl Save for PhotonCollector {
    /// Loads the fields of the photon into a vec of vecs and outputs using a table to CSV.
    fn save_data(&self, path: &Path) -> Result<(), Error> {
        if self.photons.iter().count() > 0 {
            let mut file = File::create(path)?;

            // To reduce the time to run, I am manually do my won CSV write, directly from this vec.
            let headings = vec![
                "pos_x",
                "pos_y",
                "pos_z",
                "dir_x",
                "dir_y",
                "dir_z",
                "wavelength",
                "power",
                "weight",
                "tof",
                "uid",
            ];
            write!(file, "{}", headings[0])?;
            for heading in headings.iter().skip(1) {
                write!(file, ",{}", heading)?;
            }
            writeln!(file)?;

            // This can potentually
            let mut pb = ProgressBar::new("Saving Photons", self.photons.iter().count());

            // Write the properties of each of the photons to the output table.
            for phot in self.photons.iter() {
                writeln!(
                    file,
                    "{},{},{},{},{},{},{},{},{},{},{:08X}",
                    phot.ray().pos().x(),
                    phot.ray().pos().y(),
                    phot.ray().pos().z(),
                    phot.ray().dir().x(),
                    phot.ray().dir().y(),
                    phot.ray().dir().z(),
                    phot.wavelength(),
                    phot.power(),
                    phot.weight(),
                    phot.tof().unwrap_or(0.0),
                    phot.uid().encode(),
                )?;
                pb.tick();
            }
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

#[cfg(test)]
mod tests {
    use super::PhotonCollector;
    use crate::{
        geom::Ray,
        math::{Dir3, Point3},
        phys::Photon,
    };

    #[test]
    fn test_photon_collect() {
        let mut col = PhotonCollector::new();
        let pos = Point3::new(0.0, 0.0, 0.0);
        let dir = Dir3::new(1.0, 0.0, 0.0);
        let mut test_phot = Photon::new(Ray::new(pos.clone(), dir.clone()), 5.0E-7, 1.0);

        // Now collect the photon and check that it a) was collected and b) all of the quantities were conserved.
        col.collect_photon(&mut test_phot);

        assert_eq!(col.nphoton(), 1);
        assert_eq!(*col.photons[0].ray().pos(), pos);
        assert_eq!(*col.photons[0].ray().dir(), dir);
        assert_eq!(col.photons[0].wavelength(), 5.0E-7);
        assert_eq!(col.photons[0].power(), 1.0);
    }

    #[test]
    fn test_clone_photon_collector() {
        let mut col = PhotonCollector::new();
        // This time we will get it to kill the photon when we collect it.
        col.kill_photon = true;
        let pos = Point3::new(0.0, 0.0, 0.0);
        let dir = Dir3::new(1.0, 0.0, 0.0);
        let mut test_phot = Photon::new(Ray::new(pos.clone(), dir.clone()), 5.0E-7, 1.0);

        // Now collect the photon and check that it a) was collected and b) all of the quantities were conserved.
        col.collect_photon(&mut test_phot);
        let cloned = col.clone();

        assert_eq!(cloned.nphoton(), 1);
        assert_eq!(*cloned.photons[0].ray().pos(), pos);
        assert_eq!(*cloned.photons[0].ray().dir(), dir);
        assert_eq!(cloned.photons[0].wavelength(), 5.0E-7);
        assert_eq!(cloned.photons[0].power(), 1.0);

        // Check that the photon was indeed killed.
        assert_eq!(test_phot.weight(), 0.0);
    }

    #[test]
    fn test_add_assign_photon_collector() {
        let mut col1 = PhotonCollector::new();
        let mut col2 = PhotonCollector::new();
        let pos = Point3::new(0.0, 0.0, 0.0);
        let dir = Dir3::new(1.0, 0.0, 0.0);
        let mut test_phot = Photon::new(Ray::new(pos.clone(), dir.clone()), 5.0E-7, 1.0);

        // Now collect the photon and check that everything was conserved during the clone.
        col1.collect_photon(&mut test_phot);
        col2.collect_photon(&mut test_phot);
        col1 += &col2;

        assert_eq!(col1.nphoton(), 2);
        assert_eq!(*col1.photons[1].ray().pos(), *col2.photons[0].ray().pos());
        assert_eq!(*col1.photons[1].ray().dir(), *col2.photons[0].ray().dir());
        assert_eq!(col1.photons[0].wavelength(), col2.photons[0].wavelength());
        assert_eq!(col1.photons[0].power(), col2.photons[0].power());
    }
}
