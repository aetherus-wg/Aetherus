use serde::{Deserialize, Serialize};
use crate::{
    math::Point2,
    io::output::{OutputPlane, AxisAlignedPlane},
};

#[derive(Debug, Deserialize, Serialize)]
pub struct OutputPlaneBuilder {
    boundary: (Point2, Point2),
    res: [usize; 2],
    plane: AxisAlignedPlane,
}

impl OutputPlaneBuilder {
    pub fn build(&self) -> OutputPlane {
        OutputPlane::new(self.boundary.0, self.boundary.1, self.res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_deserialise_build() {
        let input = r#"
            {
                plane: "xy",
                boundary: [[0, 0], [10, 10]],
                res: [10, 10],
            }
        "#;

        let builder: OutputPlaneBuilder = json5::from_str(input).unwrap();
        let outvol = builder.build();
        assert_eq!(outvol.pix_area(), 1.0);
    }
}
