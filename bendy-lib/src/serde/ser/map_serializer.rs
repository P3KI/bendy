use crate::serde::common::*;

/// Bencode sub-serializer for maps.
pub struct MapSerializer<'outer> {
    pub(crate) outer: &'outer mut Encoder,
    encoder: UnsortedDictEncoder,
    key: Option<Vec<u8>>,
}

impl<'outer> MapSerializer<'outer> {
    pub(crate) fn new(
        outer: &'outer mut Encoder,
        encoder: UnsortedDictEncoder,
    ) -> MapSerializer<'outer> {
        MapSerializer {
            encoder,
            outer,
            key: None,
        }
    }

    fn serialize<T>(&self, value: &T) -> Result<Vec<u8>>
    where
        T: ?Sized + Serialize,
    {
        let mut serializer = Serializer::with_max_depth(self.encoder.remaining_depth());
        value.serialize(&mut serializer)?;
        serializer.into_bytes()
    }
}

impl<'outer> SerializeMap for MapSerializer<'outer> {
    type Error = Error;
    type Ok = ();

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if self.key.is_some() {
            return Err(Error::MapSerializationCallOrder);
        }

        let mut encoded = self.serialize(key)?;

        match encoded.first() {
            Some(b'0'..=b'9') => {},
            _ => return Err(Error::ArbitraryMapKeysUnsupported),
        }

        let colon = encoded.iter().position(|b| *b == b':').unwrap();
        encoded.drain(0..colon + 1);

        self.key = Some(encoded);

        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match self.key.take() {
            Some(bytes) => {
                let encoded = self.serialize(value)?;
                self.encoder.save_pair(&bytes, encoded)?;
                Ok(())
            },
            None => Err(Error::MapSerializationCallOrder),
        }
    }

    fn end(self) -> Result<()> {
        if self.key.is_some() {
            return Err(Error::MapSerializationCallOrder);
        }
        self.outer.end_unsorted_dict(self.encoder)?;
        Ok(())
    }
}
