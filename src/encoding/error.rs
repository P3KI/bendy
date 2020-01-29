#[cfg(feature = "std")]
use std::sync::Arc;

use failure::Fail;

use crate::state_tracker::StructureError;

#[derive(Debug, Clone, Fail)]
#[fail(display = "encoding failed: {}", _0)]
pub struct Error(#[fail(cause)] pub ErrorKind);

/// An enumeration of potential errors that appear during bencode encoding.
#[derive(Debug, Clone, Fail)]
pub enum ErrorKind {
    /// Error that occurs if the serialized structure contains invalid semantics.
    #[cfg(feature = "std")]
    #[fail(display = "malformed content discovered: {}", _0)]
    MalformedContent(Arc<failure::Error>),
    /// Error that occurs if the serialized structure contains invalid semantics.
    #[cfg(not(feature = "std"))]
    #[fail(display = "malformed content discovered")]
    MalformedContent,
    /// Error in the bencode structure (e.g. a missing field end separator).
    #[fail(display = "bencode encoding corrupted")]
    StructureError(#[fail(cause)] StructureError),
}

impl Error {
    /// Raised when there is a general error while deserializing a type.
    /// The message should not be capitalized and should not end with a period.
    ///
    /// Note that, when building with no_std, this method accepts any type as
    /// its argument.
    #[cfg(feature = "std")]
    pub fn malformed_content(cause: impl Into<failure::Error>) -> Error {
        let error = Arc::new(cause.into());
        Self(ErrorKind::MalformedContent(error))
    }

    #[cfg(not(feature = "std"))]
    pub fn malformed_content<T>(_cause: T) -> Error {
        Self(ErrorKind::MalformedContent)
    }
}

impl From<StructureError> for Error {
    fn from(error: StructureError) -> Self {
        Self(ErrorKind::StructureError(error))
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self(kind)
    }
}
