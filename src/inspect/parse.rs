#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
#[cfg(feature = "std")]
use std::vec::Vec;

use crate::{
    decoding::{Decoder, DictDecoder, Error as DecodeError, ListDecoder, Object},
    inspect::*,
    state_tracker::StructureError,
};

impl<'ser> TryFrom<&'ser [u8]> for Inspectable<'ser> {
    type Error = DecodeError;

    /// Decodes bencode data into an Inspectable
    fn try_from(buf: &'ser [u8]) -> Result<Inspectable<'ser>, Self::Error> {
        let mut decoder = Decoder::new(buf);
        let obj = decoder
            .next_object()?
            .ok_or_else(|| StructureError::UnexpectedEof)?;
        Self::try_from(obj)
    }
}

impl<'obj, 'ser> TryFrom<Object<'obj, 'ser>> for Inspectable<'ser> {
    type Error = DecodeError;

    fn try_from(object: Object<'obj, 'ser>) -> Result<Inspectable<'ser>, DecodeError> {
        Ok(match object {
            Object::List(ld) => Inspectable::List(InList::try_from(ld)?),
            Object::Dict(dd) => Inspectable::Dict(InDict::try_from(dd)?),
            Object::Integer(i) => Inspectable::Int(InInt::new(i)),
            Object::Bytes(b) => Inspectable::String(InString::new(b)),
        })
    }
}

impl<'obj, 'ser> TryFrom<ListDecoder<'obj, 'ser>> for InList<'ser> {
    type Error = DecodeError;

    fn try_from(mut ld: ListDecoder<'obj, 'ser>) -> Result<Self, Self::Error> {
        let mut items: Vec<Inspectable<'ser>> = Vec::new();
        while let Some(item) = ld.next_object()? {
            items.push(item.try_into()?);
        }

        Ok(InList { items })
    }
}

impl<'obj, 'ser> TryFrom<DictDecoder<'obj, 'ser>> for InDict<'ser> {
    type Error = DecodeError;

    fn try_from(mut dd: DictDecoder<'obj, 'ser>) -> Result<Self, Self::Error> {
        let mut items: Vec<InDictEntry> = Vec::new();

        while let Some((k, v)) = dd.next_pair()? {
            items.push(InDictEntry {
                key: Inspectable::String(InString::new(k)),
                value: v.try_into()?,
            })
        }

        Ok(InDict { items })
    }
}
