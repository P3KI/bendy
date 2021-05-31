#[cfg(feature = "std")]
use std::sync::Arc;

use snafu::Snafu;

use crate::state_tracker;

/// An enumeration of potential errors that appear during bencode encoding.
#[derive(Debug, Clone, Snafu)]
#[non_exhaustive]
pub enum Error {
    /// Error that occurs if the serialized structure contains invalid semantics.
    #[cfg(feature = "std")]
    #[snafu(display("malformed content discovered: {}", source))]
    MalformedContent {
        source: Arc<dyn std::error::Error + Send + Sync>,
    },

    /// Error that occurs if the serialized structure contains invalid semantics.
    #[cfg(not(feature = "std"))]
    #[snafu(display("malformed content discovered"))]
    MalformedContent,

    /// Error in the bencode structure (e.g. a missing field end separator).
    #[snafu(display("bencode encoding corrupted"))]
    StructureError {
        source: state_tracker::StructureError,
    },
}

impl Error {
    /// Raised when there is a general error while deserializing a type.
    /// The message should not be capitalized and should not end with a period.
    ///
    /// Note that, when building with no_std, this method accepts any type as
    /// its argument.
    #[cfg(feature = "std")]
    pub fn malformed_content<SourceT>(source: SourceT) -> Self
    where
        SourceT: std::error::Error + Send + Sync + 'static,
    {
        let error = Arc::new(source);
        Error::MalformedContent { source: error }
    }

    #[cfg(not(feature = "std"))]
    pub fn malformed_content<T>(_cause: T) -> Self {
        Self::from(ErrorKind::MalformedContent)
    }
}

impl From<state_tracker::StructureError> for Error {
    fn from(error: state_tracker::StructureError) -> Self {
        Error::StructureError { source: error }
    }
}
