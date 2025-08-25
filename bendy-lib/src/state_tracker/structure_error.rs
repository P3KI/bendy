#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
};
#[cfg(not(feature = "std"))]
use core::fmt::Display;
#[cfg(feature = "std")]
use std::fmt::Display;

use thiserror::Error;

/// An encoding or decoding error
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Error)]
pub enum StructureError {
    /// Wrong type of token detected.
    #[error("Saw the wrong type of token: {state}")]
    InvalidState { state: String },

    /// Keys were not sorted.
    #[error("Keys were not sorted")]
    UnsortedKeys,

    /// EOF reached to early.
    #[error("Reached EOF in the middle of a message")]
    UnexpectedEof,

    /// Unexpected characters detected.
    #[error("Malformed number of unexpected character: {unexpected}")]
    SyntaxError { unexpected: String },

    /// Exceeded the recursion limit.
    #[error("Maximum nesting depth exceeded")]
    NestingTooDeep,
}

impl StructureError {
    pub fn unexpected(expected: impl Display, got: char, offset: usize) -> Self {
        StructureError::SyntaxError {
            unexpected: format!("Expected {expected}, got {got:?} at offset {offset}"),
        }
    }

    pub fn invalid_state(expected: impl Display) -> Self {
        StructureError::InvalidState {
            state: expected.to_string(),
        }
    }
}
