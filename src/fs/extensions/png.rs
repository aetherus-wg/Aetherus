//! Png writing.

use crate::{err::Error, fs::Save, diag::Image};
use ndarray::{Array2, ShapeBuilder};
use palette::{Pixel, Srgba};
use png::{BitDepth, ColorType, Encoder};
use slice_of_array::SliceFlatExt;
use std::{fs::File, io::BufWriter, path::Path};

impl Save for Image {
    #[inline]
    fn save_data(&self, path: &Path) -> Result<(), Error> {
        let res = (self.pixels().shape()[0], self.pixels().shape()[1]);
        let mut data: Array2<[u8; 4]> = Array2::from_elem((res.0, res.1).f(), [0; 4]);
        for xi in 0..res.0 {
            for yi in 0..res.1 {
                let col = self.pixels()[(xi, yi)];
                data[(xi, res.1 - yi - 1)] = Srgba::from_linear(col).into_format().into_raw();
            }
        }

        let file = File::create(path)?;
        let w = BufWriter::new(file);

        let mut encoder = Encoder::new(w, res.0 as u32, res.1 as u32);
        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(BitDepth::Eight);
        let mut writer = encoder.write_header()?;

        writer.write_image_data(data.into_raw_vec().flat())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diag::{Image, Colour};
    use ndarray::arr2;

    #[test]
    fn test_save() {
        let pixel = Colour::new(0.0, 0.0, 0.0, 1.0); 
        let img = Image::new(arr2(&[[pixel, pixel], [pixel, pixel]]));
        let path = Path::new("test.png");
        let res = img.save(&path);
        assert!(res.is_ok());

        // Clean-up the written image. 
        std::fs::remove_file(path).unwrap();
    }
}