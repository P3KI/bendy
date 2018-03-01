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

pub mod decoder;
