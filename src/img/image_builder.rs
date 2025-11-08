use std::fmt::{Display, Formatter};
use serde::{Serialize, Deserialize};
use crate::{
    fmt_report,
    img::{Image, Colour}
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageBuilder {
    res: [usize; 2],
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