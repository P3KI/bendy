//! Encodes and decodes bencoded structures.
//!
//! The decoder is explicitly designed to be zero-copy as much as possible, and to not
//! accept any sort of invalid encoding in any mode (including non-canonical encodings)
//!
//! The encoder is likewise designed to ensure that it only produces valid structures.
#![cfg_attr(feature = "cargo-clippy", allow(needless_return))]
#![cfg_attr(not(test), warn(missing_docs))]

extern crate failure;
#[macro_use]
extern crate failure_derive;

#[cfg(test)]
extern crate regex;

pub mod decoder;
pub mod encoder;
mod state_tracker;

/// An encoding or decoding error
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Fail)]
pub enum Error {
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

impl Error {
    fn unexpected(expected: &str, got: char, offset: usize) -> Self {
        Error::SyntaxError(format!(
            "Expected {}, got {:?} at offset {}",
            expected, got, offset
        ))
    }

    fn invalid_state(expected: &str) -> Self {
        Error::InvalidState(expected.to_owned())
    }
}
