use serde::{Deserialize, Serialize};
use crate::{
    geom::Cube,
    math::Vec3,
    io::output::{OutputVolume, OutputParameter},
};

/// Configuration for the OutputVolume. 
/// Importantly this can be serialised / deserialised using serde, so that this
/// can be built from a JSON configuration file. 
#[derive(Serialize, Deserialize, Debug)]
pub struct OutputVolumeBuilder {
    boundary: (Vec3, Vec3),
    res: [usize; 3],
    param: OutputParameter,
}

impl OutputVolumeBuilder {
    pub fn build(&self) -> OutputVolume {
        let bound = Cube::new(self.boundary.0.data().into(), self.boundary.1.data().into());
        OutputVolume::new(bound, self.res, self.param.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_deserialise_build() {
        let input = r#"
            {
                boundary: [[0, 0, 0], [10, 10, 10]],
                res: [10, 10, 10],
                param: "energy",
            }
        "#;

        let builder: OutputVolumeBuilder = json5::from_str(input).unwrap();
        let outvol = builder.build();
        assert_eq!(outvol.cell_volume(), 1.0);
    }
}
