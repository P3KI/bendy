use crate::serde::common::*;

use serde::ser::SerializeStruct;

/// Bencode sub-serializer for structs.
pub struct StructSerializer<'outer> {
    pub(crate) outer: &'outer mut Serializer,
    contents: BTreeMap<&'static str, Vec<u8>>,
    remaining_depth: usize,
}

impl<'outer> StructSerializer<'outer> {
    pub(crate) fn new(
        outer: &'outer mut Serializer,
        remaining_depth: usize,
    ) -> StructSerializer<'outer> {
        StructSerializer {
            contents: BTreeMap::new(),
            remaining_depth,
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
        if self.contents.contains_key(key) {
            panic!("bendy::StructSerializer::serialize_field: serialize_field called with duplicate field name")
        }

        let mut serializer = Serializer::with_max_depth(self.remaining_depth);
        value.serialize(&mut serializer)?;
        let value_bytes = serializer.into_bytes()?;

        self.contents.insert(key, value_bytes);
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.outer.emit_struct(self.contents)
    }
}
