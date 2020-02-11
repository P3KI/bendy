use crate::serde::common::*;

/// Bencode sub-serializer for structs.
pub struct StructSerializer<'outer> {
    pub(crate) outer: &'outer mut Serializer,
    encoder: UnsortedDictEncoder,
}

impl<'outer> StructSerializer<'outer> {
    pub(crate) fn new(
        outer: &'outer mut Serializer,
        remaining_depth: usize,
    ) -> StructSerializer<'outer> {
        StructSerializer {
            encoder: UnsortedDictEncoder::new(remaining_depth),
            outer,
        }
    }
}

impl<'outer> SerializeStruct for StructSerializer<'outer> {
    type Error = Error;
    type Ok = ();

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut serializer = Serializer::with_max_depth(self.encoder.remaining_depth());
        value.serialize(&mut serializer)?;
        let value_bytes = serializer.into_bytes()?;

        self.encoder.save_pair(key.as_bytes(), value_bytes)?;

        Ok(())
    }

    fn end(self) -> Result<()> {
        let contents = self.encoder.done()?;
        self.outer.emit_struct(contents)
    }
}
