//! Serde bencode serialization.

use crate::serde::common::*;

mod struct_serializer;

pub use struct_serializer::StructSerializer;

/// Serialize an instance of `T` to bencode
pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    let mut serializer = Serializer::new();
    value.serialize(&mut serializer)?;
    serializer.into_bytes()
}

/// A serde Bencode serializer
pub struct Serializer {
    encoder: Encoder,
}

impl Serializer {
    /// Create a new `Serializer`
    pub fn new() -> Self {
        Serializer {
            encoder: Encoder::new(),
        }
    }

    /// Create a new `Serializer` with a given maximum serialization depth
    pub fn with_max_depth(max_depth: usize) -> Serializer {
        Serializer {
            encoder: Encoder::new().with_max_depth(max_depth),
        }
    }

    /// Consume this `Serializer`, returning the encoded bencode
    pub fn into_bytes(self) -> Result<Vec<u8>> {
        Ok(self.encoder.get_output()?)
    }
}

impl<'a> serde::ser::Serializer for &'a mut Serializer {
    type Error = Error;
    type Ok = ();
    type SerializeMap = Self;
    type SerializeSeq = Self;
    type SerializeStruct = StructSerializer<'a>;
    type SerializeStructVariant = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;

    fn serialize_bool(self, _v: bool) -> Result<()> {
        Err(Error::unsupported_type("bool"))
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.encoder.emit(v)?;
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.encoder.emit(v)?;
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.encoder.emit(v)?;
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.encoder.emit(v)?;
        Ok(())
    }

    fn serialize_i128(self, v: i128) -> Result<()> {
        self.encoder.emit(v)?;
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.encoder.emit(v)?;
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.encoder.emit(v)?;
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.encoder.emit(v)?;
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.encoder.emit(v)?;
        Ok(())
    }

    fn serialize_u128(self, v: u128) -> Result<()> {
        self.encoder.emit(v)?;
        Ok(())
    }

    fn serialize_f32(self, _v: f32) -> Result<()> {
        Err(Error::unsupported_type("f32"))
    }

    fn serialize_f64(self, _v: f64) -> Result<()> {
        Err(Error::unsupported_type("f64"))
    }

    fn serialize_char(self, _v: char) -> Result<()> {
        Err(Error::unsupported_type("char"))
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.serialize_bytes(v.as_bytes())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.encoder.emit_bytes(v)?;
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        Err(Error::unsupported_type("Option"))
    }

    fn serialize_some<T>(self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::unsupported_type("Option"))
    }

    fn serialize_unit(self) -> Result<()> {
        Err(Error::unsupported_type("()"))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Err(Error::unsupported_type("unit struct"))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        Err(Error::unsupported_type("enum unit variant"))
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::unsupported_type("enum newtype variant"))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.encoder.emit_token(Token::List)?;
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        self.encoder.emit_token(Token::List)?;
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.encoder.emit_token(Token::List)?;
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::unsupported_type("enum tuple variant"))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::unsupported_type("map"))
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        let encoder = self.encoder.begin_unsorted_dict()?;
        Ok(StructSerializer::new(&mut self.encoder, encoder))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::unsupported_type("enum struct variant"))
    }
}

impl<'a> SerializeSeq for &'a mut Serializer {
    type Error = Error;
    type Ok = ();

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.encoder.emit_token(Token::End)?;
        Ok(())
    }
}

impl<'a> SerializeTuple for &'a mut Serializer {
    type Error = Error;
    type Ok = ();

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.encoder.emit_token(Token::End)?;
        Ok(())
    }
}

impl<'a> SerializeTupleStruct for &'a mut Serializer {
    type Error = Error;
    type Ok = ();

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.encoder.emit_token(Token::End)?;
        Ok(())
    }
}

impl<'a> SerializeMap for &'a mut Serializer {
    type Error = Error;
    type Ok = ();

    fn serialize_key<T>(&mut self, _key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn serialize_value<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<()> {
        unreachable!()
    }
}

impl<'a> SerializeTupleVariant for &'a mut Serializer {
    type Error = Error;
    type Ok = ();

    fn serialize_field<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<()> {
        unreachable!()
    }
}

impl<'a> SerializeStructVariant for &'a mut Serializer {
    type Error = Error;
    type Ok = ();

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<()> {
        unreachable!()
    }
}
