use std::fmt::{Display, Formatter};
use log::warn;
use serde::Deserialize;
use ndarray::Array3;
use crate::{
    access, clone, err::Error, fmt_report, io::output::OrientBuilder, ord::{Link, Name, Set, cartesian::{X, Y}}, phys::Light, tools::Range
};

#[derive(Debug, Deserialize, Clone)]
pub struct CcdBuilder {
    res: [usize; 2],
    range: Option<[f64; 2]>,
    bins: usize,
    width: f64,
    height: Option<f64>,
    orient: OrientBuilder,
}

impl<'a> Link<'a, Light<'a>> for CcdBuilder {
    type Inst = CcdBuilder;

    fn requires(&self) -> Vec<Name> {
        vec![]
    }

    fn link(mut self, set: &Set<Light>) -> Result<Self::Inst, Error> {
        for light in set.values() {
            if self.range.is_none() {
                self.range = Some([light.spec().min(), light.spec().max()]);
            } else {
                let range = self.range.as_mut().unwrap();
                range[0] = range[0].min(light.spec().min());
                range[1] = range[1].max(light.spec().max());
            }
        }
        Ok(self)
    }
}

impl CcdBuilder {
    access!(orient: OrientBuilder);
    clone!(bins: usize);
    clone!(width: f64);

    pub fn height(&self) -> f64 {
        if let Some(height) = self.height {
            height
        } else {
            warn!("Height of Ccd not provided. Inferring from width and resolution.");
            self.width * (self.res[Y] as f64) / (self.res[X] as f64)
        }
    }

    pub fn range(&self) -> Result<Range, Error> {
        if let Some(range) = self.range {
            Ok(Range::new(range[0], range[1]))
        } else {
            Err(Error::Linking("Range must be provided for Ccd if not linked to any light sources.".to_string()))
        }
    }

    pub fn build(&self) -> Array3<f64> {
        Array3::zeros([self.res[X], self.res[Y], self.bins])
    }
}

impl Display for CcdBuilder {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        fmt_report!(fmt, format!("{} x {}", self.res[0], self.res[1]), "resolution");
        fmt_report!(fmt, self.bins, "bins");
        Ok(())
    }
}
