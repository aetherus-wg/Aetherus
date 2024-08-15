use std::fmt::{Display, Formatter};
use serde::{Serialize, Deserialize};
use crate::{fmt_report, io::output::PhotonCollector};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct PhotonCollectorBuilder {
    kill_photons: Option<bool>,
}

impl PhotonCollectorBuilder {
    pub fn build(&self) -> PhotonCollector {
        let mut photcol = PhotonCollector::new();
        photcol.kill_photon = match self.kill_photons {
            Some(kp) => kp,
            None => false
        };
        photcol
    }
}

impl Display for PhotonCollectorBuilder {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        let kill_str = match self.kill_photons {
            Some(kf) => kf,
            None => false,
        };
        writeln!(fmt, "PhotonCollector: ")?;
        fmt_report!(fmt, kill_str, "kill on collect");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use json5;

    #[test]
    fn test_new() {
        let photcol = PhotonCollectorBuilder::default().build();
        assert_eq!(photcol.kill_photon, false);
        assert_eq!(photcol.nphoton(), 0);
    }

    #[test]
    fn test_deserialise_default() {
        let input = "{}";
        let photcolbuild: PhotonCollectorBuilder = json5::from_str(&input).unwrap();
        let photcol = photcolbuild.build();
        assert_eq!(photcol.kill_photon, false);
        assert_eq!(photcol.nphoton(), 0);
    }

    #[test]
    fn test_deserialise_kill_photons() {
        let input = "{ kill_photons: true }";
        let photcolbuild: PhotonCollectorBuilder = json5::from_str(&input).unwrap();
        let photcol = photcolbuild.build();
        assert_eq!(photcol.kill_photon, true);
        assert_eq!(photcol.nphoton(), 0);
    }
}