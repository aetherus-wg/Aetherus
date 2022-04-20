use crate::{err::Error, fs::File, math::stat::SphericalCdf};
use std::{path::Path};

impl File for SphericalCdf {
    #[inline]
    fn load(path: &Path) -> Result<Self, Error> {
        // Load the photometric web from the file into a SphericalCDF
        let lid = lidrs::photweb::PhotometricWebBuilder::from_file(path).build()?;
        let cdf: SphericalCdf = lid.into();
        Ok(cdf)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use tempfile::tempdir;

    use crate::{fs::File, math::stat::SphericalCdf};

    const IES_STR: &str = "IESNA: LM-63-2002 
[TEST] ABC1234 
[TESTLAB] ABC Laboratories 
[ISSUEDATE] 18-FEB-2001 
[MANUFAC] Aardvark lighting Inc. 
[LUMCAT] SKYVIEW 123-XYZ-abs-400 
[LUMINAIRE] Wide beam flood to be used without tilt 
[LAMPCAT] MH-400-CLEAR 
[LAMP] 400 Watt Metal Halide 
[BALLASTCAT] Global 16G6031-17R 
[BALLAST] 400W 277V MH Magnetic 
[MAINTCAT] 4 
[OTHER] This luminaire is useful as an indirect flood 
[MORE] and to reduce light pollution in down light applications. 
[LAMPPOSITION] 0,0 
[SEARCH] POLLUTION SPORTS INDIRECT 
[_NEMATYPE] 4h x 6v 
[_PRICE] Make us an offer 
TILT=INCLUDE
1
13
0 15 30 45 60 75 90 105 120 135 150 165 180
1.0 .95 .94 .90 .88 .87 .98 .87 .88 .90 .94 .95 1.0
1 50000 1 5 3 1 1 .5 .6 0
1.0 1.0 495
0 22.5 45 67.5 90
0 45 90
100000 50000 25000 10000 5000
100000 35000 16000 8000 3000
100000 20000 10000 5000 1000
";

    #[test]
    fn test_convert_photweb_to_cdf() {
        // First write to a temporary file so that we can run the conversion from scratch.
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("iesna-2002-demo.ies");
        let mut file = std::fs::File::create(&file_path).unwrap();
        let _ = file.write_all(IES_STR.as_bytes());

        // Now attempt to read.
        match SphericalCdf::load(file_path.as_path()) {
            Ok(cdf) => {
                // output to file for analysis.
                let _ = cdf.azimuth_cdf().cdf_to_file("azim.cdf");
                let _ = cdf.azimuth_cdf().pdf_to_file("azim.pdf");
                for (ipl, pl) in cdf.planes().iter().enumerate() {
                    let _ = pl.cdf().cdf_to_file(&format!("plane{}.cdf", ipl));
                    let _ = pl.cdf().pdf_to_file(&format!("plane{}.pdf", ipl));
                }

                // Now check that we have the correct number of planes.
                assert_eq!(cdf.planes().iter().count(), 8);

                // Now test the sampling of the CDF
                let mut rng = rand::thread_rng();
                let mut test_file = std::fs::File::create("samples.dat").unwrap();
                for _ in 0..100_000 {
                    let (azim, pol) = cdf.sample(&mut rng);
                    let _ = write!(test_file, "{}\t{}\n", azim, pol);
                }
            }
            Err(e) => {
                assert!(false, "{:?}", &e);
            }
        };

        let _ = dir.close();
    }
}
