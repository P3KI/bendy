//! Serde bencode serialization and deserialization.
//!
//! The Serde data model contains a number of types which have no native bencode
//! representation. Serializing and deserializing these types is currently
//! unsupported:
//! - `()`
//! - `HashMap` and `BTreeMap`
//! - `Option`
//! - `bool`
//! - `char`
//! - `f32` and `f64`
//! - enums
//! - unit structs
//!
//! In addition, the current implementation is not self-describing, so
//! deserialization relying on  `serde::de::Deserializer::deserialize_any` is
//! unsupported.

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

    use std::collections::BTreeMap;

    use super::{de::from_bytes, ser::to_bytes};

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
        #[serde(transparent)]
        struct Borrowed<'bytes> {
            #[serde(with = "serde_bytes")]
            bytes: &'bytes [u8],
        }

        case_borrowed(Borrowed { bytes: &[1, 2, 3] }, b"3:\x01\x02\x03");
    }

    #[test]
    fn newtype_struct() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
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
        struct Foo(String, u32, i32);

        case(Foo("hello".to_string(), 1, -100), "l5:helloi1ei-100ee");
    }

    #[test]
    fn struct_test() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
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
        struct Foo {
            fac: u8,
            fb: u8,
        }

        case(Foo { fac: 0, fb: 1 }, "d3:faci0e2:fbi1ee");
    }

    #[test]
    fn unsupported_bool_serialize() {
        assert_matches!(to_bytes(&true), Err(Error::UnsupportedType("bool")));
    }

    #[test]
    fn unsupported_bool_deserialize() {
        assert_matches!(from_bytes::<bool>(b""), Err(Error::UnsupportedType("bool")));
    }

    #[test]
    fn unsupported_f32_deserialize() {
        assert_matches!(from_bytes::<f32>(b""), Err(Error::UnsupportedType("f32")));
    }

    #[test]
    fn unsupported_f32_serialize() {
        assert_matches!(to_bytes(&0f32), Err(Error::UnsupportedType("f32")));
    }

    #[test]
    fn unsupported_f64_deserialize() {
        assert_matches!(from_bytes::<f64>(b""), Err(Error::UnsupportedType("f64")));
    }

    #[test]
    fn unsupported_f64_serialize() {
        assert_matches!(to_bytes(&0f64), Err(Error::UnsupportedType("f64")));
    }

    #[test]
    fn unsupported_option_deserialize() {
        assert_matches!(
            from_bytes::<Option<u8>>(b""),
            Err(Error::UnsupportedType("Option"))
        );
    }

    #[test]
    fn unsupported_some_serialize() {
        assert_matches!(to_bytes(&Some(0)), Err(Error::UnsupportedType("Option")));
    }

    #[test]
    fn unsupported_none_serialize() {
        assert_matches!(
            to_bytes::<Option<u8>>(&None),
            Err(Error::UnsupportedType("Option"))
        );
    }

    #[test]
    fn unsupported_unit_deserialize() {
        assert_matches!(from_bytes::<()>(b""), Err(Error::UnsupportedType("()")));
    }

    #[test]
    fn unsupported_unit_serialize() {
        assert_matches!(to_bytes(&()), Err(Error::UnsupportedType("()")));
    }

    #[test]
    fn unsupported_unit_struct_deserialize() {
        #[derive(Deserialize, Debug)]
        struct Foo;
        assert_matches!(
            from_bytes::<Foo>(b""),
            Err(Error::UnsupportedType("unit struct"))
        );
    }

    #[test]
    fn unsupported_unit_struct_serialize() {
        #[derive(Serialize)]
        struct Foo;
        assert_matches!(to_bytes(&Foo), Err(Error::UnsupportedType("unit struct")));
    }

    #[test]
    fn unsupported_char_deserialize() {
        assert_matches!(from_bytes::<char>(b""), Err(Error::UnsupportedType("char")));
    }

    #[test]
    fn unsupported_char_serialize() {
        assert_matches!(to_bytes(&'a'), Err(Error::UnsupportedType("char")));
    }

    #[test]
    fn unsupported_map_deserialize() {
        assert_matches!(
            from_bytes::<BTreeMap<u8, u8>>(b""),
            Err(Error::UnsupportedType("map"))
        );
    }

    #[test]
    fn unsupported_map_serialize() {
        let map: BTreeMap<u8, u8> = BTreeMap::new();
        assert_matches!(to_bytes(&map), Err(Error::UnsupportedType("map")));
    }

    #[test]
    fn unsupported_enum_deserialize() {
        #[derive(Deserialize, Debug)]
        enum Foo {}
        assert_matches!(from_bytes::<Foo>(b""), Err(Error::UnsupportedType("enum")));
    }

    #[test]
    fn unsupported_any_deserialize() {
        #[serde(untagged)]
        #[derive(Deserialize, Debug)]
        pub(crate) enum Foo {
            A { _x: char },
            B { _x: String },
        }
        assert_matches!(
            from_bytes::<Foo>(b""),
            Err(Error::UnsupportedSelfDescribing)
        );
    }
}
