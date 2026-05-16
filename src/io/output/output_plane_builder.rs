use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use crate::{
    fmt_report, io::output::{AxisAlignedPlane, OutputPlane}, math::Point2, ord::{Build, cartesian::{X, Y}}
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OutputPlaneBuilder {
    boundary: (Point2, Point2),
    res: [usize; 2],
    plane: AxisAlignedPlane,
}

impl Build for OutputPlaneBuilder {
    type Inst = OutputPlane;
    type MetaInfo = ();
    fn build(self, _id: ()) -> Result<Self::Inst, crate::err::Error> {
        Ok(OutputPlane::new(self.boundary.0, self.boundary.1, self.res, self.plane.clone()))
    }
}

#[cfg(test)]
mod tests {
    use crate::err::Error;

    use super::*;

    #[test]
    fn new_deserialise_build() -> Result<(), Error> {
        let input = r#"
            {
                plane: "xy",
                boundary: [[0, 0], [10, 10]],
                res: [10, 10],
            }
        "#;

        let builder: OutputPlaneBuilder = json5::from_str(input).unwrap();
        let outvol = builder.build(())?;
        assert_eq!(outvol.pix_area(), 1.0);

        Ok(())
    }
}

impl Display for OutputPlaneBuilder {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        writeln!(fmt, "...")?;
        fmt_report!(fmt, "...", "boundary");
        fmt_report!(fmt, format!("[{}, {}]", self.boundary.0.x(), self.boundary.0.y()), "mins");
        fmt_report!(fmt, format!("[{}, {}]", self.boundary.1.x(), self.boundary.1.y()), "maxs");
        fmt_report!(
            fmt,
            &format!("[{} x {}]", self.res[X], self.res[Y]),
            "resolution"
        );
        fmt_report!(fmt, self.plane, "plane");
        Ok(())
    }
}
