#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap, rc::Rc, string::String, vec::Vec};

#[cfg(feature = "std")]
use std::{
    collections::{BTreeMap, HashMap},
    hash::{BuildHasher, Hash},
    rc::Rc,
};

use crate::{
    decoding::{Decoder, Error, Object},
    encoding::AsString,
    state_tracker::StructureError,
};

///Basic trait for bencode based value deserialization.
pub trait FromBencode {
    /// Maximum allowed depth of nested structures before the decoding should be aborted.
    const EXPECTED_RECURSION_DEPTH: usize = 2048;

    /// Deserialize an object from its byte representation.
    fn from_bencode(bytes: &[u8]) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut decoder = Decoder::new(bytes).with_max_depth(Self::EXPECTED_RECURSION_DEPTH);
        let object = decoder.next_object()?;

        object.map_or(
            Err(Error::from(StructureError::UnexpectedEof)),
            Self::decode_bencode_object,
        )
    }

    /// Deserialize an object from its intermediate bencode representation.
    fn decode_bencode_object(object: Object) -> Result<Self, Error>
    where
        Self: Sized;
}

macro_rules! impl_from_bencode_for_integer {
    ($($type:ty)*) => {$(
        impl FromBencode for $type {
            const EXPECTED_RECURSION_DEPTH: usize = 0;

            fn decode_bencode_object(object: Object) -> Result<Self, Error>
            where
                Self: Sized,
            {
                let content = object.try_into_integer()?;
                let number = content.parse::<$type>()?;

                Ok(number)
            }
        }
    )*}
}

impl_from_bencode_for_integer!(u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize);

impl<ContentT: FromBencode> FromBencode for Vec<ContentT> {
    const EXPECTED_RECURSION_DEPTH: usize = ContentT::EXPECTED_RECURSION_DEPTH + 1;

    fn decode_bencode_object(object: Object) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut list = object.try_into_list()?;
        let mut results = Vec::new();

        while let Some(object) = list.next_object()? {
            let item = ContentT::decode_bencode_object(object)?;
            results.push(item);
        }

        Ok(results)
    }
}

impl FromBencode for String {
    const EXPECTED_RECURSION_DEPTH: usize = 0;

    fn decode_bencode_object(object: Object) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let content = object.try_into_bytes()?;
        let content = String::from_utf8(content.to_vec())?;

        Ok(content)
    }
}

impl<K, V> FromBencode for BTreeMap<K, V>
where
    K: FromBencode + Ord,
    V: FromBencode,
{
    const EXPECTED_RECURSION_DEPTH: usize = V::EXPECTED_RECURSION_DEPTH + 1;

    fn decode_bencode_object(object: Object) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut dict = object.try_into_dictionary()?;
        let mut result = BTreeMap::default();

        while let Some((key, value)) = dict.next_pair()? {
            let key = K::decode_bencode_object(Object::Bytes(key))?;
            let value = V::decode_bencode_object(value)?;

            result.insert(key, value);
        }

        Ok(result)
    }
}

#[cfg(feature = "std")]
impl<K, V, H> FromBencode for HashMap<K, V, H>
where
    K: FromBencode + Hash + Eq,
    V: FromBencode,
    H: BuildHasher + Default,
{
    const EXPECTED_RECURSION_DEPTH: usize = V::EXPECTED_RECURSION_DEPTH + 1;

    fn decode_bencode_object(object: Object) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut dict = object.try_into_dictionary()?;
        let mut result = HashMap::default();

        while let Some((key, value)) = dict.next_pair()? {
            let key = K::decode_bencode_object(Object::Bytes(key))?;
            let value = V::decode_bencode_object(value)?;

            result.insert(key, value);
        }

        Ok(result)
    }
}

impl<T: FromBencode> FromBencode for Rc<T> {
    const EXPECTED_RECURSION_DEPTH: usize = T::EXPECTED_RECURSION_DEPTH;

    fn decode_bencode_object(object: Object) -> Result<Self, Error>
    where
        Self: Sized,
    {
        T::decode_bencode_object(object).map(Rc::new)
    }
}

impl FromBencode for AsString<Vec<u8>> {
    const EXPECTED_RECURSION_DEPTH: usize = 0;

    fn decode_bencode_object(object: Object) -> Result<Self, Error>
    where
        Self: Sized,
    {
        object.try_into_bytes().map(Vec::from).map(AsString)
    }
}

#[cfg(test)]
mod test {

    #[cfg(not(feature = "std"))]
    use alloc::{format, vec::Vec};

    use crate::encoding::AsString;

    use super::*;

    #[test]
    fn from_bencode_to_string_should_work_with_valid_input() {
        let expected_message = "hello";
        let serialized_message =
            format!("{}:{}", expected_message.len(), expected_message).into_bytes();

        let decoded_message = String::from_bencode(&serialized_message).unwrap();
        assert_eq!(expected_message, decoded_message);
    }

    #[test]
    fn from_bencode_to_as_string_should_work_with_valid_input() {
        let expected_message = "hello";
        let serialized_message =
            format!("{}:{}", expected_message.len(), expected_message).into_bytes();

        let decoded_vector = AsString::from_bencode(&serialized_message).unwrap();
        assert_eq!(expected_message.as_bytes(), &decoded_vector.0[..]);
    }

    #[test]
    #[should_panic(expected = "Num")]
    fn from_bencode_to_as_string_should_fail_for_integer() {
        AsString::<Vec<u8>>::from_bencode(&b"i1e"[..]).unwrap();
    }

    #[test]
    #[should_panic(expected = "NestingTooDeep")]
    fn from_bencode_to_as_string_should_fail_for_list() {
        AsString::<Vec<u8>>::from_bencode(&b"l1:ae"[..]).unwrap();
    }

    #[test]
    #[should_panic(expected = "NestingTooDeep")]
    fn from_bencode_to_as_string_should_fail_for_dictionary() {
        AsString::<Vec<u8>>::from_bencode(&b"d1:a1:ae"[..]).unwrap();
    }
}
