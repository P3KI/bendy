//! Serde bencode deserialization.

use crate::serde::common::*;

/// Deserialize an instance of `T` from bencode
pub fn from_bytes<'a, T>(s: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    Deserializer::from_bytes(s).deserialize()
}

/// Bencode deserializer
pub struct Deserializer<'de> {
    forbid_trailing_bytes: bool,
    tokens: Peekable<Tokens<'de>>,
}

impl<'de> Deserializer<'de> {
    /// Create a new `Deserializer` with the give byte slice
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Deserializer {
            forbid_trailing_bytes: false,
            tokens: Decoder::new(input).tokens().peekable(),
        }
    }

    /// Return an error if trailing bytes remain after deserialization
    pub fn with_forbid_trailing_bytes(mut self, forbid_trailing_bytes: bool) -> Self {
        self.forbid_trailing_bytes = forbid_trailing_bytes;
        self
    }

    /// Consume the deserializer, producing an instance of `T`
    pub fn deserialize<T>(mut self) -> Result<T, Error>
    where
        T: Deserialize<'de>,
    {
        let t = T::deserialize(&mut self)?;

        if self.forbid_trailing_bytes {
            if let Some(_) = self.tokens.next() {
                return Err(Error::TrailingBytes);
            }
        }

        Ok(t)
    }
}

impl<'de> Deserializer<'de> {
    fn next_token(&mut self) -> Result<Token<'de>> {
        match self.tokens.next() {
            Some(result) => Ok(result?),
            None => Err(Error::Decode(StructureError::UnexpectedEof.into())),
        }
    }

    fn next_integer(&mut self) -> Result<&'de str> {
        match self.next_token()? {
            Token::Num(num) => Ok(num),
            other => Err(decoding::Error::unexpected_token("Num", other.name()).into()),
        }
    }

    fn next_bytes(&mut self) -> Result<&'de [u8]> {
        match self.next_token()? {
            Token::String(bytes) => Ok(bytes),
            other => Err(decoding::Error::unexpected_token("String", other.name()).into()),
        }
    }

    fn next_string(&mut self) -> Result<&'de str> {
        let bytes = self.next_bytes()?;
        let string = str::from_utf8(bytes)?;
        Ok(string)
    }

    fn expect_list_begin(&mut self) -> Result<()> {
        match self.next_token()? {
            Token::List => Ok(()),
            other => Err(decoding::Error::unexpected_token("List", other.name()).into()),
        }
    }

    fn expect_dict_begin(&mut self) -> Result<()> {
        match self.next_token()? {
            Token::Dict => Ok(()),
            other => Err(decoding::Error::unexpected_token("Dict", other.name()).into()),
        }
    }

    fn expect_end(&mut self) -> Result<()> {
        match self.next_token()? {
            Token::End => Ok(()),
            other => Err(decoding::Error::unexpected_token("End", other.name()).into()),
        }
    }

    fn expect_empty_list(&mut self) -> Result<()> {
        self.expect_list_begin()?;
        self.expect_end()?;
        Ok(())
    }

    fn peek_end(&mut self) -> bool {
        self.peek() == Some(Token::End)
    }

    fn peek(&mut self) -> Option<Token<'de>> {
        if let Some(Ok(token)) = self.tokens.peek() {
            Some(*token)
        } else {
            None
        }
    }
}

impl<'de, 'a> serde::de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.peek() {
            Some(Token::Dict) => self.deserialize_map(visitor),
            Some(Token::String(_)) => self.deserialize_bytes(visitor),
            Some(Token::List) => self.deserialize_seq(visitor),
            Some(Token::Num(_)) => self.deserialize_i64(visitor),
            Some(Token::End) => Err(Error::Decode(StructureError::invalid_state("End").into())),
            None => Err(Error::Decode(StructureError::UnexpectedEof.into())),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.next_integer()? {
            "0" => visitor.visit_bool(false),
            "1" => visitor.visit_bool(true),
            other => Err(Error::InvalidBool(other.to_owned())),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.next_integer()?.parse()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.next_integer()?.parse()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.next_integer()?.parse()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.next_integer()?.parse()?)
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i128(self.next_integer()?.parse()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.next_integer()?.parse()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.next_integer()?.parse()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.next_integer()?.parse()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.next_integer()?.parse()?)
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u128(self.next_integer()?.parse()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.next_bytes()?;
        let bits = u32::from_be_bytes(
            bytes
                .try_into()
                .map_err(|_| Error::InvalidF32(bytes.len()))?,
        );
        let value = f32::from_bits(bits);
        visitor.visit_f32(value)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.next_bytes()?;
        let bits = u64::from_be_bytes(
            bytes
                .try_into()
                .map_err(|_| Error::InvalidF64(bytes.len()))?,
        );
        let value = f64::from_bits(bits);
        visitor.visit_f64(value)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let s: &str = self.next_string()?;
        let count = s.chars().count();
        if count != 1 {
            return Err(Error::InvalidChar(count));
        }
        visitor.visit_char(s.chars().next().unwrap())
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.next_string()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_bytes(self.next_bytes()?)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.expect_list_begin()?;
        let value = if self.peek_end() {
            visitor.visit_none()
        } else {
            visitor.visit_some(&mut *self)
        };
        self.expect_end()?;
        value
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.expect_empty_list()?;
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.expect_list_begin()?;
        let value = visitor.visit_seq(&mut *self)?;
        self.expect_end()?;
        Ok(value)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.expect_dict_begin()?;
        let value = visitor.visit_map(&mut *self)?;
        self.expect_end()?;
        Ok(value)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.expect_dict_begin()?;
        let value = visitor.visit_map(&mut *self)?;
        self.expect_end()?;
        Ok(value)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.peek() == Some(Token::Dict) {
            self.expect_dict_begin()?;
            visitor.visit_enum(self)
        } else {
            visitor.visit_enum(self.next_string()?.into_deserializer())
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

impl<'de> SeqAccess<'de> for Deserializer<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.peek_end() {
            return Ok(None);
        }
        seed.deserialize(self).map(Some)
    }
}

impl<'de> MapAccess<'de> for Deserializer<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if self.peek_end() {
            return Ok(None);
        }
        seed.deserialize(self).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }
}

impl<'de> EnumAccess<'de> for &mut Deserializer<'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self)>
    where
        V: DeserializeSeed<'de>,
    {
        Ok((seed.deserialize(&mut *self)?, self))
    }
}

impl<'de> VariantAccess<'de> for &mut Deserializer<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        let value = seed.deserialize(&mut *self)?;
        self.expect_end()?;
        Ok(value)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = serde::de::Deserializer::deserialize_seq(&mut *self, visitor)?;
        self.expect_end()?;
        Ok(value)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = serde::de::Deserializer::deserialize_map(&mut *self, visitor)?;
        self.expect_end()?;
        Ok(value)
    }
}
