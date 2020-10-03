//! Serde Serialization and Deserialization
//! =======================================
//!
//! Values can be serialized to bencode with `bendy::serde::to_bytes`, and
//! deserialized from bencode with `bendy::serde::from_bytes`:
//!
//! ```
//! use bendy::serde::{from_bytes, to_bytes};
//! use serde_ as serde;
//! use serde_derive::{Deserialize, Serialize};
//!
//! assert_eq!(to_bytes(&10).unwrap(), b"i10e");
//! assert_eq!(from_bytes::<u64>(b"i10e").unwrap(), 10);
//!
//! #[serde(crate = "serde_")]
//! #[derive(Serialize, Deserialize, Debug, PartialEq)]
//! struct Foo {
//!     bar: bool,
//! }
//!
//! assert_eq!(to_bytes(&Foo { bar: true }).unwrap(), b"d3:bari1ee");
//! assert_eq!(from_bytes::<Foo>(b"d3:bari1ee").unwrap(), Foo { bar: true });
//! ```
//!
//! Bencode Representations
//! -----------------------
//!
//! Rust types and values are represented in bencode as follows:
//!
//! - `true`: The integer value `1`.
//! - `false`: The integer value `0`.
//! - `char`: A string containing the UTF-8 encoding of the value.
//! - `f32`: Represented as a length-four bencode byte string containing the big-
//!   endian order bytes of the IEEE-754 representation of the value.
//! - `f64`: Represented as a length-eight bencode byte string containing the big-
//!   endian order bytes of the IEEE-754 representation of the value.
//! - `()`: Represented as the empty bencode list, `le`.
//! - `Some(t)`: Represented as a list containing the bencoding of `t`.
//! - `None`: Represented as the empty list.
//! - maps, including BTreeMap and HashMap: bencoded dictionaries.
//! - record structs: Represented as bencoded dictionaries with the fields of the
//!   struct represented as UTF-8 keys mapped to the bencoded serializations of the
//!   values.
//! - tuple structs: Represented as bencoded lists containing the serialized values
//!   of the fields.
//! - unit structs: Represented as the empty bencode list, `le`.
//! - enum unit variants: Represented as a string containing the name of the variant,
//! - enum newtype variants: Represented as a dict mapping the name of the variant
//!   to the value the variant contains.
//! - enum tuple variants: Represented as a dict mapping the name of the variant
//!   to a list containing the fields of the enum.
//! - enum struct variants: Represented as a dict mapping the name of the variant
//!   to the struct representation of the fields of the variant.
//! - untagged enums: Repesented as the variant value without any surrounding dictionary.
//!
//! Bencode dictionary keys may only be byte strings. For this reason, map types with
//! keys that do not serialize as byte strings are unsupported.
//!
//! Note that values of type `f32` and `f64` do not conform to bencode's canonical
//! representation rules. For example, both `f32` and `f64` support negative zero
//! values which have different bit patterns, but which represent the same logical
//! value as positive zero.
//!
//! If you require bencoded values to have canonical representations, then it is best
//! to avoid floating point values.
//!
//! Example Representations
//! -----------------------
//!
//! ```
//! use bendy::serde::to_bytes;
//! use serde::Serialize;
//! use serde_ as serde;
//! use serde_derive::Serialize;
//! use std::collections::HashMap;
//!
//! fn repr(value: impl Serialize, bencode: impl AsRef<[u8]>) {
//!     assert_eq!(to_bytes(&value).unwrap(), bencode.as_ref());
//! }
//!
//! repr(true, "i1e");
//! repr(false, "i0e");
//! repr((), "le");
//! repr('a', "1:a");
//! repr('Å', b"2:\xC3\x85");
//! repr(0, "i0e");
//! repr(-15, "i-15e");
//! repr(1.0f32, b"4:\x3F\x80\x00\x00");
//! repr(1.0f64, b"8:\x3F\xF0\x00\x00\x00\x00\x00\x00");
//!
//! let none: Option<i32> = None;
//! repr(none, "le");
//! repr(Some(0), "li0ee");
//!
//! let mut map = HashMap::new();
//! map.insert("foo", 1);
//! map.insert("bar", 2);
//! repr(map, "d3:bari2e3:fooi1ee");
//!
//! #[serde(crate = "serde_")]
//! #[derive(Serialize)]
//! struct Unit;
//! repr(Unit, "le");
//!
//! #[serde(crate = "serde_")]
//! #[derive(Serialize)]
//! struct Newtype(String);
//! repr(Newtype("foo".into()), "3:foo");
//!
//! #[serde(crate = "serde_")]
//! #[derive(Serialize)]
//! struct Tuple(bool, i32);
//! repr(Tuple(false, 100), "li0ei100ee");
//!
//! #[serde(crate = "serde_")]
//! #[derive(Serialize)]
//! struct Record {
//!     a: String,
//!     b: bool,
//! }
//!
//! repr(
//!     Record {
//!         a: "hello".into(),
//!         b: false,
//!     },
//!     "d1:a5:hello1:bi0ee",
//! );
//!
//! #[serde(crate = "serde_")]
//! #[derive(Serialize)]
//! enum Enum {
//!     Unit,
//!     Newtype(i32),
//!     Tuple(bool, i32),
//!     Struct { a: char, b: bool },
//! }
//!
//! repr(Enum::Unit, "4:Unit");
//! repr(Enum::Newtype(-1), "d7:Newtypei-1ee");
//! repr(Enum::Tuple(true, 10), "d5:Tupleli1ei10eee");
//! repr(Enum::Struct { a: 'x', b: true }, "d6:Structd1:a1:x1:bi1eee");
//!
//! #[serde(untagged)]
//! #[serde(crate = "serde_")]
//! #[derive(Serialize)]
//! enum Untagged {
//!     Foo { x: i32 },
//!     Bar { y: char },
//! }
//!
//! repr(Untagged::Foo { x: -1 }, "d1:xi-1ee");
//! repr(Untagged::Bar { y: 'z' }, "d1:y1:ze");
//! ```

mod common;

pub mod de;
pub mod error;
pub mod ser;

pub use de::{from_bytes, Deserializer};
pub use error::{Error, Result};
pub use ser::{to_bytes, Serializer};

#[cfg(test)]
mod tests {
    use super::common::*;

    use std::{collections::HashMap, fmt::Debug};

    use super::{
        de::{from_bytes, Deserializer},
        ser::to_bytes,
    };

    use serde::{de::DeserializeOwned, ser::Serialize};
    use serde_derive::{Deserialize, Serialize};

    fn case<V, B>(value: V, want: B)
    where
        V: Serialize + DeserializeOwned + PartialEq + Debug,
        B: AsRef<[u8]>,
    {
        let want = want.as_ref();

        let encoded = match to_bytes(&value) {
            Ok(have) => {
                assert_eq!(
                    have,
                    want,
                    "Expected `{}` but got `{}` when serializing `{:?}`",
                    String::from_utf8_lossy(&want),
                    String::from_utf8_lossy(&have),
                    value
                );
                have
            },
            Err(err) => panic!("Failed to serialize `{:?}`: {}", value, err),
        };

        let deserialized = match from_bytes::<V>(&encoded) {
            Ok(deserialized) => deserialized,
            Err(error) => panic!(
                "Failed to deserialize `{:?}` from `{}`: {}",
                value,
                String::from_utf8_lossy(&encoded),
                error
            ),
        };

        assert_eq!(
            deserialized, value,
            "Deserialized value != original: `{:?}` != `{:?}`",
            deserialized, value
        );
    }

    fn case_borrowed<V, B>(value: V, want: B)
    where
        V: Serialize + Debug,
        B: AsRef<[u8]>,
    {
        let want = want.as_ref();

        match to_bytes(&value) {
            Ok(have) => {
                assert_eq!(
                    have,
                    want,
                    "Expected `{}` but got `{}` when serializing `{:?}`",
                    String::from_utf8_lossy(&want),
                    String::from_utf8_lossy(&have),
                    value
                );
            },
            Err(err) => panic!("Failed to serialize `{:?}`: {}", value, err),
        }
    }

    #[test]
    fn scalar() {
        case(false, "i0e");
        case(true, "i1e");
        case(0u8, "i0e");
        case(1u8, "i1e");
        case(0u16, "i0e");
        case(1u16, "i1e");
        case(0u32, "i0e");
        case(1u32, "i1e");
        case(0u64, "i0e");
        case(1u64, "i1e");
        case(0u128, "i0e");
        case(1u128, "i1e");
        case(0usize, "i0e");
        case(1usize, "i1e");
        case(0i8, "i0e");
        case(1i8, "i1e");
        case(-1i8, "i-1e");
        case(0i16, "i0e");
        case(1i16, "i1e");
        case(-1i16, "i-1e");
        case(0i32, "i0e");
        case(1i32, "i1e");
        case(-1i32, "i-1e");
        case(0i64, "i0e");
        case(1i64, "i1e");
        case(-1i64, "i-1e");
        case(0i128, "i0e");
        case(1i128, "i1e");
        case(-1i128, "i-1e");
        case(0isize, "i0e");
        case(1isize, "i1e");
        case(-1isize, "i-1e");
    }

    #[test]
    fn f32() {
        let value = 100.100f32;
        let bytes = value.to_bits().to_be_bytes();
        let mut bencode: Vec<u8> = Vec::new();
        bencode.extend(b"4:");
        bencode.extend(&bytes);
        case(value, bencode);
    }

    #[test]
    fn f64() {
        let value = 100.100f64;
        let bytes = value.to_bits().to_be_bytes();
        let mut bencode: Vec<u8> = Vec::new();
        bencode.extend(b"8:");
        bencode.extend(&bytes);
        case(value, bencode);
    }

    #[test]
    fn unit() {
        case((), "le");
    }

    #[test]
    fn none() {
        case::<Option<u8>, &str>(None, "le");
    }

    #[test]
    fn some() {
        case(Some(0), "li0ee");
    }

    #[test]
    fn char() {
        case('a', "1:a");
        case('\u{1F9D0}', "4:\u{1F9D0}");
    }

    #[test]
    fn str() {
        case_borrowed("foo", "3:foo");
    }

    #[test]
    fn string() {
        case("foo".to_string(), "3:foo");
    }

    #[test]
    fn bytes_default() {
        let value: Vec<u8> = vec![1, 2, 3, 4];
        case(value, "li1ei2ei3ei4ee");
    }

    #[test]
    fn bytes_with_serde_bytes() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(crate = "serde_")]
        #[serde(transparent)]
        struct Owned {
            #[serde(with = "serde_bytes")]
            bytes: Vec<u8>,
        }

        case(
            Owned {
                bytes: vec![1, 2, 3],
            },
            "3:\x01\x02\x03",
        );

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(crate = "serde_")]
        #[serde(transparent)]
        struct Borrowed<'bytes> {
            #[serde(with = "serde_bytes")]
            bytes: &'bytes [u8],
        }

        case_borrowed(Borrowed { bytes: &[1, 2, 3] }, b"3:\x01\x02\x03");
    }

    #[test]
    fn map() {
        let mut map = HashMap::new();
        map.insert("foo".to_owned(), 1);
        map.insert("bar".to_owned(), 2);
        case(map, "d3:bari2e3:fooi1ee");
    }

    #[test]
    fn map_non_byte_key() {
        let mut map = HashMap::new();
        map.insert(1, 1);
        map.insert(2, 2);
        assert_matches!(to_bytes(&map), Err(Error::ArbitraryMapKeysUnsupported));
    }

    #[test]
    fn unit_struct() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(crate = "serde_")]
        struct Foo;
        case(Foo, "le");
    }

    #[test]
    fn newtype_struct() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(crate = "serde_")]
        struct Foo(u8);
        case(Foo(1), "i1e");
    }

    #[test]
    fn seq() {
        case(vec![1, 0, 1], "li1ei0ei1ee");
    }

    #[test]
    fn tuple_struct() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        #[serde(crate = "serde_")]
        struct Foo(String, u32, i32);

        case(Foo("hello".to_string(), 1, -100), "l5:helloi1ei-100ee");
    }

    #[test]
    fn record_struct() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        #[serde(crate = "serde_")]
        struct Foo {
            a: u8,
            b: String,
        }

        case(
            Foo {
                a: 1,
                b: "hello".to_string(),
            },
            "d1:ai1e1:b5:helloe",
        );
    }

    #[test]
    fn struct_field_order() {
        // Serde serializes the fields of this struct in the opposite
        // order to that mandated by bencode. This would trigger an
        // error if the struct serializer failed to correctly order
        // the fields during serialization.
        #[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
        #[serde(crate = "serde_")]
        struct Foo {
            fac: u8,
            fb: u8,
        }

        case(Foo { fac: 0, fb: 1 }, "d3:faci0e2:fbi1ee");
    }

    #[test]
    fn enum_tests() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        #[serde(crate = "serde_")]
        enum Enum {
            Unit,
            Newtype(i32),
            Tuple(bool, i32),
            Struct { a: char, b: bool },
        }

        case(Enum::Unit, "4:Unit");
        case(Enum::Newtype(-1), "d7:Newtypei-1ee");
        case(Enum::Tuple(true, 10), "d5:Tupleli1ei10eee");
        case(Enum::Struct { a: 'x', b: true }, "d6:Structd1:a1:x1:bi1eee");
    }

    #[test]
    fn untagged_enum() {
        #[serde(untagged)]
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        #[serde(crate = "serde_")]
        enum Untagged {
            Foo { x: i32 },
            Bar { y: String },
        }

        case(Untagged::Foo { x: -1 }, "d1:xi-1ee");
        case(Untagged::Bar { y: "z".into() }, "d1:y1:ze");
    }

    #[test]
    fn flatten() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        #[serde(crate = "serde_")]
        struct Foo {
            #[serde(flatten)]
            bar: Bar,
        }

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        #[serde(crate = "serde_")]
        struct Bar {
            x: i32,
        }

        case(Foo { bar: Bar { x: 1 } }, "d1:xi1ee");
    }

    #[test]
    fn invalid_bool() {
        assert_matches!(
            from_bytes::<bool>(b"i100e"),
            Err(Error::InvalidBool(ref value)) if value == "100"
        );
    }

    #[test]
    fn invalid_f32() {
        assert_matches!(from_bytes::<f32>(b"8:10000000"), Err(Error::InvalidF32(8)));
    }

    #[test]
    fn invalid_f64() {
        assert_matches!(from_bytes::<f64>(b"4:1000"), Err(Error::InvalidF64(4)));
    }

    #[test]
    fn invalid_char() {
        assert_matches!(from_bytes::<char>(b"2:00"), Err(Error::InvalidChar(2)));
    }

    #[test]
    fn trailing_bytes_forbid() {
        assert_matches!(
            Deserializer::from_bytes(b"i1ei1e")
                .with_forbid_trailing_bytes(true)
                .deserialize::<u32>(),
            Err(Error::TrailingBytes)
        );
    }

    #[test]
    fn trailing_bytes_allow() {
        assert_matches!(
            Deserializer::from_bytes(b"i1ei1e").deserialize::<u32>(),
            Ok(1)
        );
    }

    #[test]
    fn borrowed_value() {
        use crate::value::Value;
        use std::borrow::Cow;

        #[derive(Debug, Deserialize, PartialEq, Eq)]
        #[serde(crate = "serde_")]
        struct Dict<'a> {
            #[serde(borrow)]
            v: Value<'a>,
        }

        assert_eq!(
            Deserializer::from_bytes(b"d1:v3:\x01\x02\x03e")
                .deserialize::<Dict<'_>>()
                .unwrap(),
            Dict {
                v: Value::Bytes(Cow::Owned(vec![1, 2, 3]))
            },
        );
    }
}
