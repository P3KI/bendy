//! `Value`s hold arbitrary borrowed or owneed bencode data. Unlike `Objects`,
//! they can be cloned and traversed multiple times.
//!
//! `Value` implements `FromBencode`, `ToBencode`. If the `serde` feature is
//! enabled, it also implements `Serialize` and `Deserialize`.

use alloc::{
    borrow::{Cow, ToOwned},
    collections::BTreeMap,
    vec::Vec,
};

#[cfg(feature = "serde")]
use std::{
    convert::TryInto,
    fmt::{self, Formatter},
    marker::PhantomData,
};

#[cfg(feature = "serde")]
use serde_ as serde;

#[cfg(feature = "serde")]
use serde::{
    ser::{SerializeMap, SerializeSeq},
    Serialize,
};

use crate::{
    decoding::{FromBencode, Object},
    encoding::{SingleItemEncoder, ToBencode},
};

/// An owned or borrowed bencoded value.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Value<'a> {
    /// An owned or borrowed byte string
    Bytes(Cow<'a, [u8]>),
    /// A dictionary mapping byte strings to values
    Dict(BTreeMap<Cow<'a, [u8]>, Value<'a>>),
    /// A signed integer
    Integer(i64),
    /// A list of values
    List(Vec<Value<'a>>),
}

impl<'a> Value<'a> {
    /// Convert this Value into an owned Value with static lifetime
    pub fn into_owned(self) -> Value<'static> {
        match self {
            Value::Bytes(bytes) => Value::Bytes(Cow::Owned(bytes.into_owned())),
            Value::Dict(dict) => Value::Dict(
                dict.into_iter()
                    .map(|(key, value)| (Cow::Owned(key.into_owned()), value.into_owned()))
                    .collect(),
            ),
            Value::Integer(integer) => Value::Integer(integer),
            Value::List(list) => Value::List(list.into_iter().map(Value::into_owned).collect()),
        }
    }
}

impl<'a> ToBencode for Value<'a> {
    // This leaves some room for external containers.
    // TODO(#38): Change this to 0 for v0.4
    const MAX_DEPTH: usize = usize::max_value() / 4;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), crate::encoding::Error> {
        match self {
            Value::Bytes(bytes) => encoder.emit_bytes(bytes),
            Value::Dict(dict) => dict.encode(encoder),
            Value::Integer(integer) => integer.encode(encoder),
            Value::List(list) => list.encode(encoder),
        }
    }
}

impl<'a> FromBencode for Value<'a> {
    const EXPECTED_RECURSION_DEPTH: usize = <Self as ToBencode>::MAX_DEPTH;

    fn decode_bencode_object(object: Object) -> Result<Self, crate::decoding::Error> {
        match object {
            Object::Bytes(bytes) => Ok(Value::Bytes(Cow::Owned(bytes.to_owned()))),
            Object::Dict(mut decoder) => {
                let mut dict = BTreeMap::new();
                while let Some((key, value)) = decoder.next_pair()? {
                    dict.insert(
                        Cow::Owned(key.to_owned()),
                        Value::decode_bencode_object(value)?,
                    );
                }
                Ok(Value::Dict(dict))
            },
            Object::Integer(text) => Ok(Value::Integer(text.parse()?)),
            Object::List(mut decoder) => {
                let mut list = Vec::new();
                while let Some(object) = decoder.next_object()? {
                    list.push(Value::decode_bencode_object(object)?);
                }
                Ok(Value::List(list))
            },
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impls {
    use super::*;

    use serde_bytes::Bytes;

    impl<'a> Serialize for Value<'a> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::ser::Serializer,
        {
            match self {
                Value::Bytes(string) => serializer.serialize_bytes(string),
                Value::Integer(int) => serializer.serialize_i64(*int),
                Value::List(list) => {
                    let mut seed = serializer.serialize_seq(Some(list.len()))?;
                    for value in list {
                        seed.serialize_element(value)?;
                    }
                    seed.end()
                },
                Value::Dict(dict) => {
                    let mut seed = serializer.serialize_map(Some(dict.len()))?;
                    for (k, v) in dict {
                        let bytes = Bytes::new(k);
                        seed.serialize_entry(bytes, v)?;
                    }
                    seed.end()
                },
            }
        }
    }

    impl<'de: 'a, 'a> serde::de::Deserialize<'de> for Value<'a> {
        #[inline]
        fn deserialize<D>(deserializer: D) -> Result<Value<'a>, D::Error>
        where
            D: serde::de::Deserializer<'de>,
        {
            deserializer.deserialize_any(Visitor(PhantomData))
        }
    }

    struct Visitor<'a>(PhantomData<&'a ()>);

    impl<'de: 'a, 'a> serde::de::Visitor<'de> for Visitor<'a> {
        type Value = Value<'a>;

        fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
            formatter.write_str("any valid BEncode value")
        }

        fn visit_i64<E>(self, value: i64) -> Result<Value<'a>, E> {
            Ok(Value::Integer(value))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Value<'a>, E> {
            Ok(Value::Integer(value.try_into().unwrap()))
        }

        fn visit_borrowed_bytes<E>(self, value: &'de [u8]) -> Result<Value<'a>, E>
        where
            E: serde::de::Error,
        {
            Ok(Value::Bytes(Cow::Borrowed(value)))
        }

        fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Value<'a>, E>
        where
            E: serde::de::Error,
        {
            Ok(Value::Bytes(Cow::Borrowed(value.as_bytes())))
        }

        fn visit_string<E>(self, value: String) -> Result<Value<'a>, E> {
            Ok(Value::Bytes(Cow::Owned(value.into_bytes())))
        }

        fn visit_byte_buf<E>(self, value: Vec<u8>) -> Result<Value<'a>, E> {
            Ok(Value::Bytes(Cow::Owned(value)))
        }

        fn visit_seq<V>(self, mut access: V) -> Result<Value<'a>, V::Error>
        where
            V: serde::de::SeqAccess<'de>,
        {
            let mut list = Vec::new();
            while let Some(e) = access.next_element()? {
                list.push(e);
            }
            Ok(Value::List(list))
        }

        fn visit_map<V>(self, mut access: V) -> Result<Value<'a>, V::Error>
        where
            V: serde::de::MapAccess<'de>,
        {
            let mut map = BTreeMap::new();
            while let Some((k, v)) = access.next_entry::<&Bytes, _>()? {
                map.insert(Cow::Borrowed(k.as_ref()), v);
            }
            Ok(Value::Dict(map))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::{string::String, vec};

    fn case(value: Value, expected: impl AsRef<[u8]>) {
        let expected = expected.as_ref();

        let encoded = match value.to_bencode() {
            Ok(bytes) => bytes,
            Err(err) => panic!("Failed to encode `{:?}`: {}", value, err),
        };

        if encoded != expected {
            panic!(
                "Expected `{:?}` to encode as `{}`, but got `{}",
                value,
                String::from_utf8_lossy(expected),
                String::from_utf8_lossy(&encoded)
            )
        }

        let decoded = match Value::from_bencode(&encoded) {
            Ok(decoded) => decoded,
            Err(err) => panic!(
                "Failed to decode value from `{}`: {}",
                String::from_utf8_lossy(&encoded),
                err,
            ),
        };

        assert_eq!(decoded, value);

        #[cfg(feature = "serde")]
        {
            let deserialized = match crate::serde::de::from_bytes::<Value>(expected) {
                Ok(deserialized) => deserialized,
                Err(err) => panic!(
                    "Failed to deserialize value from `{}`: {}",
                    String::from_utf8_lossy(&expected),
                    err
                ),
            };

            if deserialized != value {
                panic!(
                    "Deserialize Serialize produced unexpected value: `{:?}` != `{:?}`",
                    deserialized, value
                );
            }

            let serialized = match crate::serde::ser::to_bytes(&value) {
                Ok(serialized) => serialized,
                Err(err) => panic!("Failed to serialize `{:?}`: {}", value, err),
            };

            if serialized != expected {
                panic!(
                    "Serialize Serialize produced unexpected bencode: `{:?}` != `{:?}`",
                    String::from_utf8_lossy(&serialized),
                    String::from_utf8_lossy(expected)
                );
            }
        }
    }

    #[test]
    fn bytes() {
        case(Value::Bytes(Cow::Borrowed(&[1, 2, 3])), b"3:\x01\x02\x03");
        case(Value::Bytes(Cow::Owned(vec![1, 2, 3])), b"3:\x01\x02\x03");
    }

    #[test]
    fn dict() {
        case(Value::Dict(BTreeMap::new()), "de");

        let mut dict = BTreeMap::new();
        dict.insert(Cow::Borrowed("foo".as_bytes()), Value::Integer(1));
        dict.insert(Cow::Borrowed("bar".as_bytes()), Value::Integer(2));
        case(Value::Dict(dict), "d3:bari2e3:fooi1ee");
    }

    #[test]
    fn integer() {
        case(Value::Integer(0), "i0e");
        case(Value::Integer(-1), "i-1e");
    }

    #[test]
    fn list() {
        case(Value::List(Vec::new()), "le");
        case(
            Value::List(vec![
                Value::Integer(0),
                Value::Bytes(Cow::Borrowed(&[1, 2, 3])),
            ]),
            b"li0e3:\x01\x02\x03e",
        );
    }
}
