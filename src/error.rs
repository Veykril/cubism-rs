//! Errors returned by cubism.
use std::{error, fmt, io};

use cubism_core::MocError;

/// The result type, returned by this library.
pub type CubismResult<T> = std::result::Result<T, CubismError>;

/// An error returned by this library.
#[derive(Debug)]
pub enum CubismError {
    /// A moc loading error occurred while loading a model.
    Moc(MocError),
    /// A json error occurred while serializing or deserializing a json
    /// file.
    Json(serde_json::Error),
    /// An io error occurred.
    Io(io::Error),
}

impl error::Error for CubismError {}
impl fmt::Display for CubismError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CubismError::Moc(e) => (e as &dyn fmt::Display).fmt(fmt),
            CubismError::Json(e) => (e as &dyn fmt::Display).fmt(fmt),
            CubismError::Io(e) => (e as &dyn fmt::Display).fmt(fmt),
        }
    }
}

impl From<MocError> for CubismError {
    fn from(e: MocError) -> Self {
        CubismError::Moc(e)
    }
}

impl From<serde_json::Error> for CubismError {
    fn from(e: serde_json::Error) -> Self {
        CubismError::Json(e)
    }
}

impl From<io::Error> for CubismError {
    fn from(e: io::Error) -> Self {
        CubismError::Io(e)
    }
}
