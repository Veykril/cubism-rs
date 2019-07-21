use core::fmt;
use std::error;

/// The result type, returned by this library.
pub type CubismResult<T> = std::result::Result<T, CubismError>;

/// The error type returned by all fallible functions.
#[derive(Debug)]
pub enum CubismError {
    /// The moc version of the data passed to [`Model::from_bytes`] is too old
    /// and therefore cannot be used.
    MocVersionMismatch(u32),
    /// The moc data passed to [`Model::from_bytes`] was invalid.
    InvalidMocData,
}

impl error::Error for CubismError {}

impl fmt::Display for CubismError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CubismError::MocVersionMismatch(v) => write!(
                fmt,
                "the moc version of the file is too old for the cubism core, found {}",
                v
            ),
            CubismError::InvalidMocData => write!(fmt, "the moc data is invalid"),
        }
    }
}
