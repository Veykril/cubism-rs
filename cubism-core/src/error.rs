use std::{error, fmt};

/// The result type, returned by this library.
pub type MocResult<T> = std::result::Result<T, MocError>;

/// An error returned by [`Model::from_bytes`].
///
/// [`Model::from_bytes`]: ../struct.Model.html#method.from_bytes
#[derive(Copy, Clone, Debug)]
pub enum MocError {
    /// The moc version of the data passed is too old and therefore cannot be
    /// loaded.
    MocVersionMismatch(u32),
    /// The moc data passed is not a valid moc file.
    InvalidMocData,
}

impl error::Error for MocError {}

impl fmt::Display for MocError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MocError::MocVersionMismatch(v) => write!(
                fmt,
                "the moc version of the file is too old for the cubism core lib, found {}",
                v
            ),
            MocError::InvalidMocData => write!(fmt, "the moc data is invalid"),
        }
    }
}
