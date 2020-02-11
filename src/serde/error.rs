///! Serde error and result types
use crate::serde::common::*;

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// An enumeration of potential errors that appear during serde serialiation and
/// deserialization
#[derive(Debug)]
pub enum Error {
    /// Error that occurs if a serde-related error occurs during serialization
    CustomEncode(String),
    /// Error that occurs if a serde-related error occurs during deserialization
    CustomDecode(String),
    /// Error that occurs if the serializer or deserializer encounters an unsupported type
    UnsupportedType(&'static str),
    /// Error that occurs if the deserializer is used in a way that requires a self-describing
    /// format, which is not yet supported
    UnsupportedSelfDescribing,
    /// Error that occurs if a problem is encountered during serialization
    Encode(encoding::Error),
    /// Error that occurs if a problem is encountered during deserialization
    Decode(decoding::Error),
}

impl Error {
    pub(crate) fn unsupported_type(name: &'static str) -> Error {
        Self::UnsupportedType(name)
    }
}

impl From<encoding::Error> for Error {
    fn from(encoding_error: encoding::Error) -> Self {
        Self::Encode(encoding_error)
    }
}

impl From<decoding::Error> for Error {
    fn from(decoding_error: decoding::Error) -> Self {
        Self::Decode(decoding_error)
    }
}

impl From<ParseIntError> for Error {
    fn from(parse_int_error: ParseIntError) -> Self {
        Self::Decode(parse_int_error.into())
    }
}

impl From<Utf8Error> for Error {
    fn from(utf8_error: Utf8Error) -> Self {
        Self::Decode(utf8_error.into())
    }
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::CustomEncode(msg.to_string())
    }
}

impl serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::CustomDecode(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::CustomEncode(message) => write!(f, "Serialization failed: {}", message),
            Self::CustomDecode(message) => write!(f, "Deserialization failed: {}", message),
            Self::Encode(error) => write!(f, "{}", error),
            Self::Decode(error) => write!(f, "{}", error),
            Self::UnsupportedType(name) => write!(
                f,
                "Serializing and deserializing values of type `{}` is not supported",
                name
            ),
            Self::UnsupportedSelfDescribing => write!(
                f,
                "Deserialization that requires a self-describing format is not yet supported"
            ),
        }
    }
}

impl std::error::Error for Error {}
