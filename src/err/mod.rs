//! Error handling.

use std::fmt::Debug;
use thiserror;

/// Error enumeration.
#[derive(Debug, thiserror::Error)]
//#[error(transparent)]
pub enum Error {
    /// Description error.
    #[error("Text error: {0}")]
    Text(String),
    /// Parallelisation poison.
    #[error("Parallelisation poison.")]
    Parallel,
    /// UID Ledger error
    #[error("UIDs Ledger error: {0}")]
    Ledger(String),
    /// Linking error
    #[error("Linking error: {0}")]
    Linking(String),
    /// Formatting error.
    #[error("Formatting")]
    Format(#[from] std::fmt::Error),
    /// Missing environment variable error.
    #[error("Missing env var")]
    EnvVar(#[from] std::env::VarError),
    /// File loading error.
    #[error("File loading")]
    LoadFile(#[from] std::io::Error),
    /// Integer parsing error.
    #[error("Integer parsing")]
    ParseInt(#[from] std::num::ParseIntError),
    /// Float parsing error.
    #[error("Float parsing")]
    ParseFloat(#[from] std::num::ParseFloatError),
    /// Json reading error.
    #[error("Json reading")]
    ReadJson(#[from] json5::Error),
    /// Json writing error.
    #[error("Json writing")]
    WriteJson(#[from] serde_json::Error),
    /// Png writing error.
    #[error("PNG writing")]
    WritePng(#[from] png::EncodingError),
    /// Shape error.
    #[error("Invalid array shape")]
    InvalidShape(#[from] ndarray::ShapeError),
    /// Min/max error.
    #[error("MinMax")]
    MinMax(#[from] ndarray_stats::errors::MinMaxError),
    /// NetCDF io error.
    #[error("NetCDF IO")]
    NetCdf(#[from] netcdf::Error),
    /// Lidrs Error.
    #[error("Lidrs")]
    Lidrs(#[from] lidrs::err::Error),
    /// Obj loading
    #[error("Objfile reading")]
    ObjLoad(#[from] obj::ObjError),
    /// Obj loading
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
}

unsafe impl Send for Error {}
unsafe impl Sync for Error {}

impl From<&str> for Error {
    #[inline]
    fn from(err: &str) -> Self {
        Self::Text(err.to_owned())
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Error::from(err.as_str()) // Reuse the `From<&str>` implementation
    }
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    #[inline]
    fn from(_e: std::sync::PoisonError<T>) -> Self {
        Self::Parallel
    }
}
