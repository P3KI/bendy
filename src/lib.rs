//! Encodes and decodes bencoded structures.
//!
//! The decoder is explicitly designed to be zero-copy as much as possible, and to not
//! accept any sort of invalid encoding in any mode (including non-canonical encodings)
//!
//! The encoder is likewise designed to ensure that it only produces valid structures.
#![cfg_attr(feature = "cargo-clippy", allow(needless_return))]
#![cfg_attr(not(test), warn(missing_docs))]

#[macro_use]
extern crate derive_error;
#[cfg(test)]
extern crate regex;

pub mod decoder;
pub mod encoder;
mod state_tracker;

/// An encoding or decoding error
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Error)]
pub enum Error {
    /// Saw the wrong type of token
    #[error(msg_embedded, no_from, non_std)]
    InvalidState(String),
    /// Keys were not sorted
    UnsortedKeys,
    /// Reached EOF in the middle of a message
    UnexpectedEof,
    /// Malformed number or unexpected character
    #[error(msg_embedded, no_from, non_std)]
    SyntaxError(String),
    /// Maximum nesting depth exceeded
    NestingTooDeep,
}

impl Error {
    fn unexpected(expected: &str, got: char, offset: usize) -> Self {
        Error::SyntaxError(format!(
            "Expected {}, got {:?} at offset {}",
            expected, got, offset
        ))
    }
}
