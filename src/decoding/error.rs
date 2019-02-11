use std::{
    fmt::{self, Display, Formatter},
    num, str, string,
};

use failure::Fail;

use crate::state_tracker::StructureError;

#[derive(Debug, Fail)]
pub struct Error {
    #[fail(context)]
    context: Option<String>,
    #[fail(cause)]
    error: ErrorKind,
}

/// An enumeration of potential errors that appear during bencode deserialization.
#[derive(Debug, Fail)]
pub enum ErrorKind {
    /// Error that occurs if the serialized structure contains invalid information.
    #[fail(display = "malformed content discovered: {}", _0)]
    MalformedContent(failure::Error),
    /// Error that occurs if the serialized structure is incomplete.
    #[fail(display = "missing field: {}", _0)]
    MissingField(String),
    /// Error in the bencode structure (e.g. a missing field end separator).
    #[fail(display = "bencode encoding corrupted")]
    StructureError(#[fail(cause)] StructureError),
    /// Error that occurs if the serialized structure contains an unexpected field.
    #[fail(display = "unexpected field: {}", _0)]
    UnexpectedField(String),
    /// Error through an unexpected bencode token during deserialization.
    #[fail(display = "discovered {} but expected {}", _0, _1)]
    UnexpectedToken(String, String),
}

pub trait ResultExt {
    fn context(self, context: impl std::fmt::Display) -> Self;
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self.context {
            Some(context) => write!(f, "Error: {} in {}", self.error, context),
            None => write!(f, "Error: {}", self.error),
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(error: ErrorKind) -> Self {
        Error {
            context: None,
            error,
        }
    }
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
    pub fn malformed_content(cause: impl Into<failure::Error>) -> Error {
        Self::from(ErrorKind::MalformedContent(cause.into()))
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

impl From<StructureError> for Error {
    fn from(error: StructureError) -> Self {
        Self::from(ErrorKind::StructureError(error))
    }
}

impl From<num::ParseIntError> for Error {
    fn from(error: num::ParseIntError) -> Self {
        Self::malformed_content(error)
    }
}

impl From<str::Utf8Error> for Error {
    fn from(error: str::Utf8Error) -> Self {
        Self::malformed_content(error)
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(error: string::FromUtf8Error) -> Self {
        Self::malformed_content(error)
    }
}

impl<T> ResultExt for Result<T, Error> {
    fn context(self, context: impl std::fmt::Display) -> Result<T, Error> {
        self.map_err(|err| err.context(context))
    }
}
