use serde::{Serialize, Deserialize};
use crate::img::{Image, Colour};

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