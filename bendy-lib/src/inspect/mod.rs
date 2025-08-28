//! Reflection for testing
//!
//! Provides facilities for creating an AST-like object
//! tree from any valid bencode: [`Inspectable`]. Decoding bencode into
//! an [`Inspectable`] is done using recursion, so stack size limits
//! apply.
//!
//! Use for reflection, modifying, testing, and pretty printing.
//!
//! Not recommended for use in production code.
//!
//! # Overview
//!
//! See the Reflection chapter of the [README.md][r].
//!
//! # Creating
//!
//! The [`Inspectable`] docs explain how to create an `Inspectable` to get started.
//!
//! # Displaying / Outputting
//!
//! * Use [`.emit`][Inspectable::emit] to get the proper serialized bencode representation as a
//!   byte vector.
//! * Derives [`Debug`][core::fmt::Debug].
//! * Provides [`.as_rust_string_literal`][Inspectable::as_rust_string_literal] for pretty printing.
//!   Useful when inspecting structures, hard coding test cases, etc.
//! * Provided [`Display`][core::fmt::Display] implementation is similar to `.as_rust_string_literal`,
//!   but does not insert newlines, tabs, etc. Useful for debug logs.
//!
//! # Coercion
//!
//! Mutable coercion of [`Inspectable`] to variant items: [`.string`][Inspectable::string],
//! [`.int`][`Inspectable::int`], [`.dict`][`Inspectable::dict`], [`.list`][`Inspectable::list`],
//! and [`.raw`][`Inspectable::raw`].
//!
//! Immutable coercion: [`.string_ref`][`Inspectable::string_ref`], [`.int_ref`][`Inspectable::int_ref`],
//! [`.dict_ref`][`Inspectable::dict_ref`], [`.list_ref`][`Inspectable::list_ref`],
//! and [`.raw_ref`][`Inspectable::raw_ref`].
//!
//! These coercion method names are reversed compared to the standard `_mut` idiom in Rust, because you
//! almost always want to mutate these objects, not just look at them. These methods will panic if
//! called on an [`Inspectable`] with the wrong enum variant.
//!
//! # Traversing
//!
//! Use [`inspect_path()`][ip] to create a [`PathBuilder`][pb], for use with
//! the [`.find`][Inspectable::find], [`.find_ref`][Inspectable::find_ref],
//! [`.clone_and_mutate`][Inspectable::clone_and_mutate], and [`.mutate`][Inspectable::mutate]
//! methods. These will drill down into the AST according to the
//! [`Step`][crate::inspect::traverse::Step]s in the [`Path`][pb].
//!
//! Searches in path traversal are implemented using recursion, so stack limits may
//! apply. One can also use a combination of coercion and the following methods to
//! traverse the AST without using Paths:
//!
//! * [`InList`]: [`.nth`][InList::nth], [`.nth_ref`][InList::nth_ref]
//! * [`InDict`]: [`.nth`][InDict::nth], [`.nth_ref`][InDict::nth_ref],
//!   and [`.entry`][InDict::entry], [`.entry_ref`][InDict::entry_ref].
//! * [`InDictEntry`]: [`.key`][InDictEntry::key] and [`.value`][InDictEntry::value].
//!
//! The traversal methods will all panic on failure.
//!
//! # Mutating
//!
//! There are many ways to mutate an [`Inspectable`] AST. Some of them require first coercing the
//! [`Inspectable`] to its appropriate subtype. These are all best applied using these functions:
//! [`.clone_and_mutate`][Inspectable::clone_and_mutate] and [`.mutate`][Inspectable::mutate].
//! For convenience, there is also [`.apply`][Inspectable::apply].
//!
//! Here are all the mutation methods, grouped by the type they belong to. Methods on [`Inspectable`]
//! may only be used on the type variants specified in parentheses. They will panic if used on a type
//! variant they do not support. All of these except `.replace` are also available on their respective
//! type variants.
//!
//! * [`Inspectable`]
//!     * [`.clear_content`][Inspectable::clear_content] (all)
//!     * [`.remove_entry`][`Inspectable::remove_entry`] (dicts only)
//!     * [`.remove_nth`][`Inspectable::remove_nth`] (dicts and lists)
//!     * [`.replace`][Inspectable::replace] (all)
//!     * [`.set_content_byterange`][Inspectable::set_content_byterange] (bytestrings only)
//!     * [`.truncate`][Inspectable::truncate] (bytestrings only)
//! * [`InDict`]
//!     * [`.sort`][InDict::sort]
//! * [`InInt`]
//!     * [`.set`][InInt::set]
//! * [`InString`] (bytestrings)
//!     * [`.clear_fake_length`][InString::clear_fake_length]
//!     * [`.set_fake_length`][InString::set_fake_length]
//!     * [`.set_content_str`][InString::set_content_str]
//!     * [`.set_content_string`][InString::set_content_string]
//!     * [`.set_content_u8`][InString::set_content_u8]
//!     * [`.set_content_vec`][InString::set_content_vec]
//!
//! All properties of [`Inspectable`] and its type variants are public, so you may also make your own mutators.
//!
//! [r]: https://github.com/P3KI/bendy/blob/master/README.md#reflection
//! [pb]: crate::inspect::traverse::PathBuilder
//! [ip]: crate::inspect::inspect_path

#[cfg(not(feature = "std"))]
use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec::Vec,
};
#[cfg(feature = "std")]
use std::{borrow::Cow, vec::Vec};

pub mod display;
pub mod mutate;
#[doc(hidden)]
pub mod parse;
pub mod traverse;

/// Attempt to decode a u8 buffer into an inspectable.
/// Panics on error. Use [`Inspectable::try_from`] for a
/// fallible alternative.
pub fn inspect<'ser>(buf: &'ser [u8]) -> Inspectable<'ser> {
    Inspectable::try_from(buf).expect("Could not decode buffer into inspectable")
}

/// Builds a path that can be used to traverse from an Inspectable AST.
/// [Read here][crate::inspect#traversing] for details.
pub fn inspect_path<'ser>() -> traverse::PathBuilder<'ser> {
    traverse::PathBuilder::new()
}

/// An object that represents something that bencode can
/// decode to. Usable for reflection, modification,
/// testing, and pretty printing.
///
/// See the [module][crate::inspect] documentation for usage.
///
/// # Construction
///
/// You parse bencode into an Inspectable AST like this:
/// ```
/// # use bendy;
/// # use bendy::inspect::*;
/// let bencode = b"i10e";
/// let inspectable = bendy::inspect::inspect(bencode);
/// assert_eq!(inspectable.emit(), bencode);
/// ```
///
/// Or use the [`Inspectable::try_from`] method for fallible construction.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Inspectable<'ser> {
    Int(InInt<'ser>),
    String(InString<'ser>),
    Raw(Cow<'ser, [u8]>),
    Dict(InDict<'ser>),
    List(InList<'ser>),
}

impl<'ser> Inspectable<'ser> {
    /// Construct a new [`Inspectable::Raw`]
    pub fn new_raw(buf: &'ser [u8]) -> Inspectable<'ser> {
        Inspectable::Raw(Cow::from(buf))
    }

    /// Construct a new [`Inspectable::Int`]
    pub fn new_int(i: i64) -> Inspectable<'ser> {
        let inint = InInt {
            bytes: Cow::Owned(i.to_string()),
        };
        Inspectable::Int(inint)
    }

    /// Construct a new [`Inspectable::String`]
    pub fn new_string(buf: &'ser [u8]) -> Inspectable<'ser> {
        Inspectable::String(InString::new(buf))
    }

    /// Construct a new [`Inspectable::Dict`]
    pub fn new_dict() -> Inspectable<'ser> {
        Inspectable::Dict(InDict::new())
    }

    /// Construct a new [`Inspectable::List`]
    pub fn new_list() -> Inspectable<'ser> {
        Inspectable::List(InList::new())
    }

    /// Get the serialized bencode representation of this [`Inspectable`]
    pub fn emit(&self) -> Vec<u8> {
        let mut res = Vec::new();

        fn emit_str(s: &InString, out: &mut Vec<u8>) {
            let len = s.len().to_string();
            out.extend_from_slice(len.as_bytes());
            out.push(b':');
            out.extend_from_slice(s.content())
        }

        fn dispatch(i: &Inspectable, out: &mut Vec<u8>) {
            match i {
                Inspectable::String(x) => {
                    emit_str(x, out);
                },
                Inspectable::Raw(x) => {
                    out.extend_from_slice(x);
                },
                Inspectable::Int(x) => {
                    out.push(b'i');
                    out.extend_from_slice(x.bytes.as_bytes());
                    out.push(b'e');
                },
                Inspectable::List(x) => {
                    out.push(b'l');
                    for item in x.items.iter() {
                        dispatch(item, out);
                    }
                    out.push(b'e');
                },
                Inspectable::Dict(x) => {
                    out.push(b'd');
                    for InDictEntry { key, value } in x.items.iter() {
                        dispatch(key, out);
                        dispatch(value, out);
                    }
                    out.push(b'e');
                },
            }
        }
        dispatch(self, &mut res);
        res
    }
}

/// An [`Inspectable`] AST node representing a byte string.
/// Stores the raw bytes of the bytestring, in a [`Cow`].
#[doc(alias = "bytestring")]
#[doc(alias = "byte string")]
#[doc(alias = "string")]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InString<'ser> {
    /// The bytes in this ByteString. Does not include
    /// the prefix length and prefix separator `:`.
    pub bytes: Cow<'ser, [u8]>,
    /// If set, will provide a fake length prefix when emitted.
    pub fake_length: Option<usize>,
}

/// An [`Inspectable`] AST node representing an integer.
/// Stores the raw bytes of the integer, in a [`Cow`].
#[doc(alias = "int")]
#[doc(alias = "number")]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InInt<'ser> {
    /// The raw bytes making up the number.
    /// Does not include the `i` prefix and `e` suffix.
    pub bytes: Cow<'ser, str>,
}

/// An [`Inspectable`] AST node representing a dictionary entry.
/// This only appears in the [`InDict`] node, never as a variant
/// of [`Inspectables`][Inspectable].
#[doc(alias = "dict entry")]
#[doc(alias = "dictionary entry")]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InDictEntry<'ser> {
    /// This struct field MUST contain an Inspectable::String
    /// for the resulting bencode to be valid.
    pub key: Inspectable<'ser>,
    pub value: Inspectable<'ser>,
}

/// An [`Inspectable`] AST node representing a dictionary.
#[doc(alias = "dict")]
#[doc(alias = "dictionary")]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InDict<'ser> {
    /// The entries in the dictionary.
    /// Note that bendy will NOT decode a dict
    /// if the keys are unsorted.
    pub items: Vec<InDictEntry<'ser>>,
}

/// An [`Inspectable`] AST node representing a list.
#[derive(Debug, PartialEq, Eq, Clone)]
#[doc(alias = "list")]
#[doc(alias = "vec")]
pub struct InList<'ser> {
    pub items: Vec<Inspectable<'ser>>,
}

impl<'ser> InInt<'ser> {
    pub fn new(buf: &'ser str) -> Self {
        InInt {
            bytes: Cow::from(buf),
        }
    }

    /// Returns an i64 of the value represented.
    /// Panics if this is not possible.
    #[must_use]
    pub fn as_i64(&self) -> i64 {
        self.bytes.parse().expect("Could not parse InInt as i64")
    }
}

impl<'ser> InString<'ser> {
    pub fn new(buf: &'ser [u8]) -> InString<'ser> {
        InString {
            bytes: Cow::from(buf),
            fake_length: None,
        }
    }

    /// Returns the number of bytes in the bytestring,
    /// or the fake length of the bytestring if set.
    pub fn len(&self) -> usize {
        self.fake_length.unwrap_or_else(|| self.bytes.len())
    }

    /// Returns true if there are no bytes in the bytestring,
    /// or if the fake length of the bytestring is 0 if set.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Convenience method to get an immutable reference
    /// to the bytestring's contents.
    pub fn content(&self) -> &[u8] {
        &self.bytes
    }
}

impl<'ser> InList<'ser> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        InList { items: Vec::new() }
    }
}

impl<'ser> InDict<'ser> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        InDict { items: Vec::new() }
    }
}

impl<'ser> InDictEntry<'ser> {
    /// Creates a new dict entry. If the given key is not a byte string
    /// ([`Inspectable::String`]) then bendy will not decode the resulting bencode.
    pub fn new(key: Inspectable<'ser>, value: Inspectable<'ser>) -> Self {
        InDictEntry { key, value }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_traversal_test() {
        let buf = b"\
        l\
            i99e\
            5:hello\
            d\
                3:one\
                i11e\
                3:two\
                i22e\
                5:zzzzz\
                i33e\
            e\
        e";
        let i = inspect(buf);
        let l = i.list_ref();
        assert_eq!(3, l.items.len());
        assert_eq!(99, l.nth_ref(0).int_ref().as_i64());
        assert_eq!(b"hello".as_slice(), &*l.nth_ref(1).string_ref().bytes);
        let d = i.list_ref().nth_ref(2).dict_ref();
        let t0 = d.nth_ref(0);
        let t1 = d.nth_ref(1);
        let t2 = d.entry_ref(b"zzzzz");
        assert_eq!(b"one".as_slice(), &*t0.key.string_ref().bytes);
        assert_eq!(b"two".as_slice(), &*t1.key.string_ref().bytes);
        assert_eq!(b"zzzzz".as_slice(), &*t2.key.string_ref().bytes);
        assert_eq!(11, t0.value.int_ref().as_i64());
        assert_eq!(22, t1.value.int_ref().as_i64());
        assert_eq!(33, t2.value.int_ref().as_i64());
    }

    #[test]
    fn int_modify_test() {
        let buf = b"i64e";
        let mut i = inspect(buf);
        assert_eq!(64, i.int().as_i64());
        i.int().set(32);
        assert_eq!(32, i.int().as_i64());
        assert_eq!("i32e", i.to_string().as_str());
        assert_eq!(b"i32e", i.emit().as_slice());

        let buf = b"\
        l\
            i43770e\
            d\
                3:one\
                i11e\
            e\
        e";
        let mut i = inspect(buf);
        i.list().nth(0).int().set(2);
        i.list().nth(1).dict().nth(0).value.int().set(1);
        assert_eq!(b"li2ed3:onei1eee", i.emit().as_slice());
    }

    #[test]
    fn fake_bytestring_length_test() {
        let buf = b"\
        l\
            5:hello\
            d\
                3:one\
                i11e\
            e\
        e";
        let mut i = inspect(buf);
        i.list().nth(0).string().set_fake_length(20);
        i.list()
            .nth(1)
            .dict()
            .nth(0)
            .key
            .string()
            .set_fake_length(0);
        assert_eq!(b"l20:hellod0:onei11eee", i.emit().as_slice());
        assert_eq!("l20:hellod0:onei11eee", i.to_string().as_str());
    }

    #[test]
    fn bytestring_set_content_test() {
        let buf = b"\
        l\
            5:hello\
            5:world\
            d\
                5:hello\
                5:world\
            e\
        e";
        let mut i = inspect(buf);
        i.list()
            .nth(0)
            .string()
            .set_content_string("one".to_string());
        i.list().nth(1).string().set_content_str("two");
        let entry = i.list().nth(2).dict().nth(0);
        entry.key.string().set_content_u8(b"three");
        entry.value.string().set_content_vec(Vec::from(b"four"));
        assert_eq!(b"l3:one3:twod5:three4:fouree", i.emit().as_slice());
        assert_eq!("l3:one3:twod5:three4:fouree", i.to_string().as_str());

        let mut i = inspect(b"5:hello");
        i.string().set_content_u8(b"\x00\x01");
        assert_eq!(b"2:\x00\x01", i.emit().as_slice());
        assert_eq!("2:\\x00\\x01", i.to_string().as_str());
    }
}
