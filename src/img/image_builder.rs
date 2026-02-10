use std::fmt::{Display, Formatter};
use serde::Deserialize;
use crate::{
    access,
    clone,
    fmt_report,
    img::{Colour, Image},
    io::output::OrientBuilder,
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

impl ImageBuilder {
    pub fn build(&self) -> Image {
        let background = Colour::new(0.0, 0.0, 0.0, 1.0);
        Image::new_blank(self.res, background)
    }
}

impl Display for ImageBuilder {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        fmt_report!(fmt, format!("{} x {}", self.res[0], self.res[1]), "resolution");
        Ok(())
    }
}
