//! An encoder for bencode. Guarantees that the output string is valid bencode
//!
//! # Encoding a structure
//!
//! The easiest way to encode a structure is to implement [`ToBencode`] for it. For most structures,
//! this should be very simple:
//!
//! ```
//! # use bendy::encoding::{ToBencode, SingleItemEncoder, Error};
//!
//! struct Message {
//!     foo: i32,
//!     bar: String,
//! }
//!
//! impl ToBencode for Message {
//!     // Atoms have depth one. The struct wrapper adds one level to that
//!     const MAX_DEPTH: usize = 1;
//!
//!     fn encode(&self, encoder: SingleItemEncoder) -> Result<(), Error> {
//!         encoder.emit_dict(|mut e| {
//!             // Use e to emit the values
//!             e.emit_pair(b"bar", &self.bar)?;
//!             e.emit_pair(b"foo", &self.foo)
//!         })?;
//!         Ok(())
//!     }
//! }
//! #
//! # fn main() -> Result<(), Error> {
//! #    let message = Message{
//! #      foo: 1,
//! #      bar: "quux".to_string(),
//! #    };
//! #
//! #   message.to_bencode().map(|_| ())
//! # }
//! ```
//!
//! Then, messages can be serialized using [`ToBencode::to_bencode`]:
//!
//! ```
//! # use bendy::encoding::{ToBencode, SingleItemEncoder, Error};
//! #
//! # struct Message {
//! #    foo: i32,
//! #    bar: String,
//! # }
//! #
//! # impl ToBencode for Message {
//! #     // Atoms have depth zero. The struct wrapper adds one level to that
//! #     const MAX_DEPTH: usize = 1;
//! #
//! #     fn encode(&self, encoder: SingleItemEncoder) -> Result<(), Error> {
//! #         encoder.emit_dict(|mut e| {
//! #             // Use e to emit the values. They must be in sorted order here.
//! #             // If sorting the dict first is annoying, you can also use
//! #             // encoder.emit_and_sort_dict
//! #             e.emit_pair(b"bar", &self.bar)?;
//! #             e.emit_pair(b"foo", &self.foo)
//! #         })?;
//! #         Ok(())
//! #     }
//! # }
//! #
//! # fn main() -> Result<(), Error> {
//! let message = Message {
//!     foo: 1,
//!     bar: "quux".to_string(),
//! };
//!
//! message.to_bencode()
//! #    .map(|_| ())
//! # }
//! ```
//!
//! Most primitive types already implement [`ToBencode`].
//!
//! # Nesting depth limits
//!
//! To allow this to be used on limited platforms, all implementations of [`ToBencode`] include a
//! maximum nesting depth. Atoms (integers and byte strings) are considered to have depth 0. An
//! object (a list or dict) containing only atoms has depth 1, and in general, an object has a depth
//! equal to the depth of its deepest member plus one. In some cases, an object doesn't have a
//! statically known depth. For example, ASTs may be arbitrarily nested. Such objects should
//! have their depth set to 0, and callers should construct the Encoder manually, adding an
//! appropriate buffer for the depth:
//!
//! ```
//! # use bendy::encoding::{ToBencode, Encoder, Error};
//! #
//! # type ObjectType = u32;
//! # static OBJECT: u32 = 0;
//! #
//! # fn main() -> Result<(), Error> {
//! let mut encoder = Encoder::new().with_max_depth(ObjectType::MAX_DEPTH + 10);
//!
//! encoder.emit(OBJECT)?;
//! encoder.get_output()
//! #     .map_err(Error::from)
//! #     .map(|_| ()) // ignore a success return value
//! # }
//! ```
//!
//! # Error handling
//!
//! Once an error occurs during encoding, all future calls to the same encoding stream will fail
//! early with the same error. It is not defined whether any callback or implementation of
//! [`ToBencode::encode`] is called before returning an error; such callbacks should
//! respond to failure by bailing out as quickly as possible.
//!
//! Not all values in [`Error`] can be caused by an encoding operation. Specifically, you only need
//! to worry about [`UnsortedKeys`] and [`NestingTooDeep`].
//!
//! [`ToBencode::encode`]: self::ToBencode::encode
//! [`UnsortedKeys`]: self::Error#UnsortedKeys
//! [`NestingTooDeep`]: self::Error#NestingTooDeep

mod encoder;
mod error;
mod printable_integer;
mod to_bencode;

pub use self::{
    encoder::{Encoder, SingleItemEncoder, SortedDictEncoder, UnsortedDictEncoder},
    error::{Error, ErrorKind},
    printable_integer::PrintableInteger,
    to_bencode::{AsString, ToBencode},
};
