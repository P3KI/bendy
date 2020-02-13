//! Encodes and decodes bencoded structures.
//!
//! The decoder is explicitly designed to be zero-copy as much as possible, and to not
//! accept any sort of invalid encoding in any mode (including non-canonical encodings)
//!
//! The encoder is likewise designed to ensure that it only produces valid structures.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(all(test, feature = "serde"))]
#[macro_use]
mod assert_matches;

pub mod decoding;
pub mod encoding;
pub mod state_tracker;

#[cfg(feature = "serde")]
pub mod serde;

pub mod value;
