use std::fmt::{Display, Formatter};
use serde::Deserialize;
use crate::{
    access,
    clone,
    err::Error,
    fmt_report,
    img::{Colour, Image},
    io::output::OrientBuilder,
    ord::Build
};

#[derive(Debug, Deserialize, Clone)]
pub struct ImageBuilder {
    res: [usize; 2],
    // FIXME: The size can be infered from surface describing this sensor
    width: f64,
    height: f64,
    orient: OrientBuilder,
}

impl ImageBuilder {
    access!(orient: OrientBuilder);
    clone!(width: f64);
    clone!(height: f64);
}

impl Build for ImageBuilder {
    type Inst = Image;
    type MetaInfo = ();
    fn build(self, _id: ()) -> Result<Self::Inst, Error> {
        let background = Colour::new(0.0, 0.0, 0.0, 1.0);
        Ok(Image::new_blank(self.res, background))
    }
}

impl Display for ImageBuilder {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        fmt_report!(fmt, format!("{} x {}", self.res[0], self.res[1]), "resolution");
        Ok(())
    }
}
