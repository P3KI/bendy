#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
};
#[cfg(not(feature = "std"))]
use core::fmt::Display;
#[cfg(feature = "std")]
use std::fmt::Display;

use snafu::Snafu;

/// An encoding or decoding error
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Snafu)]
pub enum StructureError {
    /// Wrong type of token detected.
    #[snafu(display("Saw the wrong type of token: {}", state))]
    InvalidState { state: String },

    /// Keys were not sorted.
    #[snafu(display("Keys were not sorted"))]
    UnsortedKeys,

    /// EOF reached to early.
    #[snafu(display("Reached EOF in the middle of a message"))]
    UnexpectedEof,

    /// Unexpected characters detected.
    #[snafu(display("Malformed number of unexpected character: {}", unexpected))]
    SyntaxError { unexpected: String },

    /// Exceeded the recursion limit.
    #[snafu(display("Maximum nesting depth exceeded"))]
    NestingTooDeep,
}

impl StructureError {
    pub fn unexpected(expected: impl Display, got: char, offset: usize) -> Self {
        StructureError::SyntaxError {
            unexpected: format!("Expected {}, got {:?} at offset {}", expected, got, offset),
        }
    }

    pub fn invalid_state(expected: impl Display) -> Self {
        StructureError::InvalidState {
            state: expected.to_string(),
        }
    }
}
