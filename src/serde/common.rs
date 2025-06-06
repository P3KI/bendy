/// Standard library
pub(crate) use std::{
    convert::TryInto,
    fmt::{self, Display, Formatter},
    iter::Peekable,
    num::ParseIntError,
    str::{self, Utf8Error},
};

pub(crate) use serde_ as serde;

/// Dependencies
pub(crate) use serde::{
    Deserialize,
    de::{
        DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess, Visitor,
    },
    ser::{
        Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
        SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
    },
};

/// Structs and enums
pub(crate) use crate::{
    decoding::{self, Decoder, Tokens},
    encoding::{self, Encoder, UnsortedDictEncoder},
    serde::{Error, Result, ser::Serializer},
    state_tracker::{StructureError, Token},
};
