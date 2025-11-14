use nalgebra::Rotation2;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
/// Two-dimensional rotation.
pub struct Rot2 {
    data: Rotation2<f64>,
}

impl Rot2 {
    pub fn new(angle: f64) -> Self {
        Rot2 {
            data: Rotation2::new(angle),
        }
    }
}

impl From<Rotation2<f64>> for Rot2 {
    #[inline]
    fn from(r: Rotation2<f64>) -> Self {
        Self { data: r }
    }
}

impl std::fmt::Display for Rot2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}
