#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
};
#[cfg(not(feature = "std"))]
use core::fmt::Display;
#[cfg(feature = "std")]
use std::fmt::Display;

use failure::Fail;

/// An encoding or decoding error
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Fail)]
pub enum StructureError {
    #[fail(display = "Saw the wrong type of token: {}", _0)]
    /// Wrong type of token detected.
    InvalidState(String),
    #[fail(display = "Keys were not sorted")]
    /// Keys were not sorted.
    UnsortedKeys,
    #[fail(display = "Reached EOF in the middle of a message")]
    /// EOF reached to early.
    UnexpectedEof,
    #[fail(display = "Malformed number of unexpected character: {}", _0)]
    /// Unexpected characters detected.
    SyntaxError(String),
    #[fail(display = "Maximum nesting depth exceeded")]
    /// Exceeded the recursion limit.
    NestingTooDeep,
}

impl StructureError {
    pub fn unexpected(expected: impl Display, got: char, offset: usize) -> Self {
        StructureError::SyntaxError(format!(
            "Expected {}, got {:?} at offset {}",
            expected, got, offset
        ))
    }

    pub fn invalid_state(expected: impl Display) -> Self {
        StructureError::InvalidState(expected.to_string())
    }
}
