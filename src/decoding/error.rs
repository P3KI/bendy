#[cfg(not(feature = "std"))]
use alloc::{str::Utf8Error, string::FromUtf8Error};
#[cfg(not(feature = "std"))]
use core::num::ParseIntError;

use alloc::{
    format,
    string::{String, ToString},
};
use core::fmt::{self, Display, Formatter};

#[cfg(feature = "std")]
use std::{error::Error as StdError, sync::Arc};

use failure::Fail;

use crate::state_tracker::StructureError;

#[derive(Debug, Clone, Fail)]
pub struct Error {
    #[fail(context)]
    context: Option<String>,
    #[fail(cause)]
    error: ErrorKind,
}

/// An enumeration of potential errors that appear during bencode deserialization.
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
    /// Error that occurs if the serialized structure is incomplete.
    #[fail(display = "missing field: {}", _0)]
    MissingField(String),
    /// Error in the bencode structure (e.g. a missing field end separator).
    #[fail(display = "bencode encoding corrupted ({})", _0)]
    StructureError(#[fail(cause)] StructureError),
    /// Error that occurs if the serialized structure contains an unexpected field.
    #[fail(display = "unexpected field: {}", _0)]
    UnexpectedField(String),
    /// Error through an unexpected bencode token during deserialization.
    #[fail(display = "discovered {} but expected {}", _0, _1)]
    UnexpectedToken(String, String),
}

pub trait ResultExt {
    fn context(self, context: impl Display) -> Self;
}

impl Error {
    pub fn context(mut self, context: impl Display) -> Self {
        if let Some(current) = self.context.as_mut() {
            *current = format!("{}.{}", context, current);
        } else {
            self.context = Some(context.to_string());
        }

        self
    }

    /// Raised when there is a general error while deserializing a type.
    /// The message should not be capitalized and should not end with a period.
    #[cfg(feature = "std")]
    pub fn malformed_content(cause: impl Into<failure::Error>) -> Error {
        let error = Arc::new(cause.into());
        Self::from(ErrorKind::MalformedContent(error))
    }

    #[cfg(not(feature = "std"))]
    pub fn malformed_content<T>(_cause: T) -> Error {
        Self::from(ErrorKind::MalformedContent)
    }

    /// Returns a `Error::MissingField` which contains the name of the field.
    pub fn missing_field(field_name: impl Display) -> Error {
        Self::from(ErrorKind::MissingField(field_name.to_string()))
    }

    /// Returns a `Error::UnexpectedField` which contains the name of the field.
    pub fn unexpected_field(field_name: impl Display) -> Error {
        Self::from(ErrorKind::UnexpectedField(field_name.to_string()))
    }

    /// Returns a `Error::UnexpectedElement` which contains a custom error message.
    pub fn unexpected_token(expected: impl Display, discovered: impl Display) -> Error {
        Self::from(ErrorKind::UnexpectedToken(
            expected.to_string(),
            discovered.to_string(),
        ))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self.context {
            Some(context) => write!(f, "Error: {} in {}", self.error, context),
            None => write!(f, "Error: {}", self.error),
        }
    }
}

impl From<StructureError> for Error {
    fn from(error: StructureError) -> Self {
        Self::from(ErrorKind::StructureError(error))
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self {
            context: None,
            error: kind,
        }
    }
}

#[cfg(not(feature = "std"))]
impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Self {
        Self::malformed_content(err)
    }
}

#[cfg(not(feature = "std"))]
impl From<Utf8Error> for Error {
    fn from(err: Utf8Error) -> Self {
        Self::malformed_content(err)
    }
}

#[cfg(not(feature = "std"))]
impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        Self::malformed_content(err)
    }
}

#[cfg(feature = "std")]
impl<T: StdError + Send + Sync + 'static> From<T> for Error {
    fn from(error: T) -> Self {
        Self::malformed_content(error)
    }
}

impl<T> ResultExt for Result<T, Error> {
    fn context(self, context: impl Display) -> Result<T, Error> {
        self.map_err(|err| err.context(context))
    }
}
