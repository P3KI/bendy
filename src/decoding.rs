//! Decodes a bencoded struct
//!
//! # Basic decoding
//! For any decoding process, first we need to create a decoder:
//!
//! ```
//! # use bendy::decoding::{Decoder};
//! #
//! # let buf: &[u8] = b"d3:fooi1ee";
//! let _decoder = Decoder::new(buf);
//! ```
//!
//! Decoders have a depth limit to prevent resource exhaustion from hostile inputs. By default, it's
//! set high enough for most structures that you'd encounter when prototyping, but for production
//! use, not only may it not be enough, but the higher the depth limit, the more stack space an
//! attacker can cause your program to use, so we recommend setting the bounds tightly:
//!
//! ```
//! # use bendy::decoding::{Decoder};
//! #
//! # let buf: &[u8] = b"d3:fooi1ee";
//! let _decoder = Decoder::new(buf).with_max_depth(3);
//! ```
//!
//! Atoms (integers and strings) have depth zero, and lists and dicts have a depth equal to the
//! depth of their deepest member plus one. As an special case, an empty list or dict has depth 1.
//!
//! Now, you can start reading objects:
//!
//! ```
//! # use bendy::decoding::{Decoder,Object};
//! #
//! # fn decode_list(_: bendy::decoding::ListDecoder) {}
//! # fn decode_dict(_: bendy::decoding::DictDecoder) {}
//! #
//! # let buf: &[u8] = b"d3:fooi1ee";
//! # let mut decoder = Decoder::new(buf);
//! #
//! match decoder.next_object().unwrap() {
//!     None => (), // EOF
//!     Some(Object::List(d)) => decode_list(d),
//!     Some(Object::Dict(d)) => decode_dict(d),
//!     Some(Object::Integer(_)) => (), // integer, as a string
//!     Some(Object::Bytes(_)) => (),   // A raw bytestring
//! };
//! ```
//!
//! # Error handling
//!
//! Once an error is encountered, the decoder won't try to muddle through it; instead, every future
//! call to the decoder will return the same error. This behaviour can be used to check the syntax
//! of an input object without fully decoding it:
//!
//! ```
//! # use bendy::decoding::Decoder;
//! #
//! fn syntax_check(buf: &[u8]) -> bool {
//!     let mut decoder = Decoder::new(buf);
//!     decoder.next_object().ok(); // ignore the return value of this
//!     return decoder.next_object().is_ok();
//! }
//! #
//! # assert!(syntax_check(b"i18e"));
//! ```

mod decoder;
mod error;
mod from_bencode;
mod object;

pub use self::{
    decoder::{Decoder, DictDecoder, ListDecoder, Tokens},
    error::{Error, ErrorKind, ResultExt},
    from_bencode::FromBencode,
    object::Object,
};
