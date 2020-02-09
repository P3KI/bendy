///! Serde error and result types
use crate::serde::common::*;

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// An enumeration of potential errors that appear during serde serialiation and
/// deserialization
#[derive(Debug)]
pub enum Error {
    /// Error that occurs if a map with a key type which does not serialize to
    /// a byte string is encountered
    ArbitraryMapKeysUnsupported,
    /// Error that occurs if methods on MapSerializer are called out of order
    MapSerializationCallOrder,
    /// Error that occurs if a bool is deserialized from an integer value other
    /// than `0` or `1`
    InvalidBool(String),
    /// Error that occurs if an f32 is deserialized from an string of length other
    /// than 4
    InvalidF32(usize),
    /// Error that occurs if an f64 is deserialized from an string of length other
    /// than 8
    InvalidF64(usize),
    /// Error that occurs if a char is deserialized from a string containing more
    /// than one character
    InvalidChar(usize),
    /// Error that occurs if a serde-related error occurs during serialization
    CustomEncode(String),
    /// Error that occurs if a serde-related error occurs during deserialization
    CustomDecode(String),
    /// Error that occurs if a problem is encountered during serialization
    Encode(encoding::Error),
    /// Error that occurs if a problem is encountered during deserialization
    Decode(decoding::Error),
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
            Self::InvalidBool(value) => write!(f, "Invalid integer value for bool: `{}`", value),
            Self::InvalidF32(length) => {
                write!(f, "Invalid length byte string value for f32: {}", length)
            },
            Self::InvalidF64(length) => {
                write!(f, "Invalid length byte string value for f64: {}", length)
            },
            Self::InvalidChar(length) => {
                write!(f, "Invalid length string value for char: {}", length)
            },
            Self::ArbitraryMapKeysUnsupported => write!(
                f,
                "Maps with key types that do not serialize to byte strings are unsupported",
            ),
            Self::MapSerializationCallOrder => {
                write!(f, "Map serialization methods called out of order")
            },
        }
    }
}

impl std::error::Error for Error {}
