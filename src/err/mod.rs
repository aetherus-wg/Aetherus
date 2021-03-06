//! Error handling.

use std::fmt::{Debug, Formatter};

/// Error enumeration.
pub enum Error {
    /// Description error.
    Text(String),
    /// Parallelisation poison.
    Parallel,
    /// Formatting error.
    Format(std::fmt::Error),
    /// Missing environment variable error.
    EnvVar(std::env::VarError),
    /// File loading error.
    LoadFile(std::io::Error),
    /// Integer parsing error.
    ParseInt(std::num::ParseIntError),
    /// Float parsing error.
    ParseFloat(std::num::ParseFloatError),
    /// Json reading error.
    ReadJson(json5::Error),
    /// Json writing error.
    WriteJson(serde_json::Error),
    /// Png writing error.
    WritePng(png::EncodingError),
    /// Shape error.
    InvalidShape(ndarray::ShapeError),
    /// Min/max error.
    MinMax(ndarray_stats::errors::MinMaxError),
    /// NetCDF io error.
    NetCdf(netcdf::error::Error),
    /// Lidrs Error.
    Lidrs(lidrs::err::Error),
}

macro_rules! impl_from_for_err {
    ($enum:path, $error:ty) => {
        impl From<$error> for Error {
            #[inline]
            fn from(e: $error) -> Self {
                $enum(e)
            }
        }
    };
}

impl From<&str> for Error {
    #[inline]
    fn from(err: &str) -> Self {
        Self::Text(err.to_owned())
    }
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    #[inline]
    fn from(_e: std::sync::PoisonError<T>) -> Self {
        Self::Parallel
    }
}

impl_from_for_err!(Self::Format, std::fmt::Error);
impl_from_for_err!(Self::EnvVar, std::env::VarError);
impl_from_for_err!(Self::LoadFile, std::io::Error);
impl_from_for_err!(Self::ParseInt, std::num::ParseIntError);
impl_from_for_err!(Self::ParseFloat, std::num::ParseFloatError);
impl_from_for_err!(Self::ReadJson, json5::Error);
impl_from_for_err!(Self::WriteJson, serde_json::Error);
impl_from_for_err!(Self::InvalidShape, ndarray::ShapeError);
impl_from_for_err!(Self::MinMax, ndarray_stats::errors::MinMaxError);
impl_from_for_err!(Self::NetCdf, netcdf::error::Error);
impl_from_for_err!(Self::WritePng, png::EncodingError);
impl_from_for_err!(Self::Lidrs, lidrs::err::Error);

impl Debug for Error {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(
            fmt,
            "{} error: `{}`",
            match *self {
                Self::Text { .. } => "Text string",
                Self::Parallel { .. } => "Parallelisation poison",
                Self::Format { .. } => "Formatting",
                Self::EnvVar { .. } => "Environment variable",
                Self::LoadFile { .. } => "File loading",
                Self::ParseInt { .. } => "Integer parsing",
                Self::ParseFloat { .. } => "Float parsing",
                Self::ReadJson { .. } => "Json reading",
                Self::WriteJson { .. } => "Json writing",
                Self::WritePng { .. } => "PNG writing",
                Self::InvalidShape { .. } => "Invalid array shape",
                Self::MinMax { .. } => "MinMax",
                Self::NetCdf { .. } => "NetCDF IO",
                Self::Lidrs { .. } => "Lidrs",
            },
            match *self {
                Self::Text { 0: ref err } => format!("{:?}", err),
                Self::Parallel => "Parallelisation fail".to_owned(),
                Self::Format { 0: ref err } => format!("{:?}", err),
                Self::EnvVar { 0: ref err } => format!("{:?}", err),
                Self::LoadFile { 0: ref err } => format!("{:?}", err),
                Self::ParseInt { 0: ref err } => format!("{:?}", err),
                Self::ParseFloat { 0: ref err } => format!("{:?}", err),
                Self::ReadJson { 0: ref err } => format!("{:?}", err),
                Self::WriteJson { 0: ref err } => format!("{:?}", err),
                Self::WritePng { 0: ref err } => format!("{:?}", err),
                Self::InvalidShape { 0: ref err } => format!("{:?}", err),
                Self::MinMax { 0: ref err } => format!("{:?}", err),
                Self::NetCdf { 0: ref err } => format!("{:?}", err),
                Self::Lidrs { 0: ref err } => format!("{:?}", err),
            }
        )
    }
}
