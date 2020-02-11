#[cfg(not(feature = "std"))]
use alloc::{
    borrow::ToOwned,
    collections::BTreeMap,
    format,
    string::{String, ToString},
    vec::Vec,
};
#[cfg(feature = "std")]
use std::{collections::BTreeMap, vec::Vec};

use crate::{
    encoding::{Error, PrintableInteger, ToBencode},
    state_tracker::{StateTracker, StructureError, Token},
};

/// The actual encoder. Unlike the decoder, this is not zero-copy, as that would
/// result in a horrible interface
#[derive(Default, Debug)]
pub struct Encoder {
    state: StateTracker<Vec<u8>, Error>,
    output: Vec<u8>,
}

impl Encoder {
    /// Create a new encoder
    pub fn new() -> Self {
        <Self as Default>::default()
    }

    /// Set the max depth of the encoded object
    #[must_use]
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.state.set_max_depth(max_depth);
        self
    }

    /// Emit a single token to the encoder
    pub(crate) fn emit_token(&mut self, token: Token) -> Result<(), Error> {
        self.state.check_error()?;
        self.state.observe_token(&token)?;
        match token {
            Token::List => self.output.push(b'l'),
            Token::Dict => self.output.push(b'd'),
            Token::String(s) => {
                // Writing to a vec can't fail
                let length = s.len().to_string();
                self.output.extend_from_slice(length.as_bytes());
                self.output.push(b':');
                self.output.extend_from_slice(s);
            },
            Token::Num(num) => {
                // Alas, this doesn't verify that the given number is valid
                self.output.push(b'i');
                self.output.extend_from_slice(num.as_bytes());
                self.output.push(b'e');
            },
            Token::End => self.output.push(b'e'),
        }

        Ok(())
    }

    /// Emit an arbitrary encodable object
    pub fn emit<E: ToBencode>(&mut self, value: E) -> Result<(), Error> {
        self.emit_with(|e| value.encode(e))
    }

    /// Emit a single object using an encoder
    pub fn emit_with<F>(&mut self, value_cb: F) -> Result<(), Error>
    where
        F: FnOnce(SingleItemEncoder) -> Result<(), Error>,
    {
        let mut value_written = false;
        let ret = value_cb(SingleItemEncoder {
            encoder: self,
            value_written: &mut value_written,
        });

        self.state.latch_err(ret)?;

        if !value_written {
            return self
                .state
                .latch_err(Err(Error::from(StructureError::invalid_state(
                    "No value was emitted",
                ))));
        }

        Ok(())
    }

    /// Emit an integer
    pub fn emit_int<T: PrintableInteger>(&mut self, value: T) -> Result<(), Error> {
        // This doesn't use emit_token, as that would require that I write the integer to a
        // temporary buffer and then copy it to the output; writing it directly saves at
        // least one memory allocation
        self.state.check_error()?;
        // We observe an int here, as we need something that isn't a string (and therefore
        // possibly valid as a key) but we also want to require as few state transitions as
        // possible (for performance)
        self.state.observe_token(&Token::Num(""))?;
        self.output.push(b'i');
        self.output.extend_from_slice(value.to_string().as_bytes());
        self.output.push(b'e');
        Ok(())
    }

    /// Emit a string
    pub fn emit_str(&mut self, value: &str) -> Result<(), Error> {
        self.emit_token(Token::String(value.as_bytes()))
    }

    /// Emit a byte array
    pub fn emit_bytes(&mut self, value: &[u8]) -> Result<(), Error> {
        self.emit_token(Token::String(value))
    }

    /// Emit a dictionary where you know that the keys are already
    /// sorted.  The callback must emit key/value pairs to the given
    /// encoder in sorted order.  If the key/value pairs may not be
    /// sorted, [`emit_unsorted_dict`] should be used instead.
    ///
    /// [`emit_unsorted_dict`]: SingleItemEncoder::emit_unsorted_dict
    ///
    /// Example:
    ///
    /// ```
    /// # use bendy::encoding::{Encoder, Error};
    /// #
    /// # fn main() -> Result<(), Error>{
    /// let mut encoder = Encoder::new();
    /// encoder.emit_dict(|mut e| {
    ///     e.emit_pair(b"a", "foo")?;
    ///     e.emit_pair(b"b", 2)
    /// })
    /// # }
    /// ```
    pub fn emit_dict<F>(&mut self, content_cb: F) -> Result<(), Error>
    where
        F: FnOnce(SortedDictEncoder) -> Result<(), Error>,
    {
        self.emit_token(Token::Dict)?;
        content_cb(SortedDictEncoder { encoder: self })?;
        self.emit_token(Token::End)
    }

    /// Emit an arbitrary list. The callback should emit the contents
    /// of the list to the given encoder.
    ///
    /// E.g., to emit the list `[1,2,3]`, you would write
    ///
    /// ```
    /// # use bendy::encoding::{Encoder, Error};
    /// # fn main() -> Result<(), Error> {
    /// let mut encoder = Encoder::new();
    /// encoder.emit_list(|e| {
    ///     e.emit_int(1)?;
    ///     e.emit_int(2)?;
    ///     e.emit_int(3)
    /// })
    /// # }
    /// ```
    pub fn emit_list<F>(&mut self, list_cb: F) -> Result<(), Error>
    where
        F: FnOnce(&mut Encoder) -> Result<(), Error>,
    {
        self.emit_token(Token::List)?;
        list_cb(self)?;
        self.emit_token(Token::End)
    }

    /// Emit a dictionary that may have keys out of order. This will write the dict
    /// values to temporary memory, then sort them before adding them to the serialized
    /// stream
    ///
    /// Example.
    ///
    /// ```
    /// # use bendy::encoding::{Encoder, Error};
    /// #
    /// # fn main() -> Result<(), Error> {
    /// let mut encoder = Encoder::new();
    /// encoder.emit_and_sort_dict(|e| {
    ///     // Unlike in the example for Encoder::emit_dict(), these keys aren't sorted
    ///     e.emit_pair(b"b", 2)?;
    ///     e.emit_pair(b"a", "foo")
    /// })
    /// # }
    /// ```
    pub fn emit_and_sort_dict<F>(&mut self, content_cb: F) -> Result<(), Error>
    where
        F: FnOnce(&mut UnsortedDictEncoder) -> Result<(), Error>,
    {
        let mut encoder = self.begin_unsorted_dict()?;

        content_cb(&mut encoder)?;

        self.end_unsorted_dict(encoder)
    }

    /// Return the encoded string, if all objects written are complete
    pub fn get_output(mut self) -> Result<Vec<u8>, Error> {
        self.state.observe_eof()?;
        Ok(self.output)
    }

    pub(crate) fn begin_unsorted_dict(&mut self) -> Result<UnsortedDictEncoder, Error> {
        // emit the dict token so that a pre-existing state error is reported early
        self.emit_token(Token::Dict)?;

        Ok(UnsortedDictEncoder::new(self.state.remaining_depth()))
    }

    pub(crate) fn end_unsorted_dict(&mut self, encoder: UnsortedDictEncoder) -> Result<(), Error> {
        let content = encoder.done()?;

        for (k, v) in content {
            self.emit_bytes(&k)?;
            // We know that the output is a single object by construction
            self.state.observe_token(&Token::Num(""))?;
            self.output.extend_from_slice(&v);
        }

        self.emit_token(Token::End)?;

        Ok(())
    }
}

/// An encoder that can only encode a single item. See [`Encoder`]
/// for usage examples; the only difference between these classes is
/// that `SingleItemEncoder` can only be used once.
pub struct SingleItemEncoder<'a> {
    encoder: &'a mut Encoder,
    /// Whether we attempted to write a value to the encoder. The value
    /// of the referent of this field is meaningless if the encode method
    /// failed.
    value_written: &'a mut bool,
}

impl<'a> SingleItemEncoder<'a> {
    /// Emit an arbitrary encodable object
    pub fn emit<E: ToBencode + ?Sized>(self, value: &E) -> Result<(), Error> {
        value.encode(self)
    }

    /// Emit a single object using an encoder
    pub fn emit_with<F>(self, value_cb: F) -> Result<(), Error>
    where
        F: FnOnce(SingleItemEncoder) -> Result<(), Error>,
    {
        value_cb(self)
    }

    /// Emit an integer
    pub fn emit_int<T: PrintableInteger>(self, value: T) -> Result<(), Error> {
        *self.value_written = true;
        self.encoder.emit_int(value)
    }

    /// Emit a string
    pub fn emit_str(self, value: &str) -> Result<(), Error> {
        *self.value_written = true;
        self.encoder.emit_str(value)
    }

    /// Emit a byte array
    pub fn emit_bytes(self, value: &[u8]) -> Result<(), Error> {
        *self.value_written = true;
        self.encoder.emit_bytes(value)
    }

    /// Emit an arbitrary list
    pub fn emit_list<F>(self, list_cb: F) -> Result<(), Error>
    where
        F: FnOnce(&mut Encoder) -> Result<(), Error>,
    {
        *self.value_written = true;
        self.encoder.emit_list(list_cb)
    }

    /// Emit a sorted dictionary. If the input dictionary is unsorted, this will return an error.
    pub fn emit_dict<F>(self, content_cb: F) -> Result<(), Error>
    where
        F: FnOnce(SortedDictEncoder) -> Result<(), Error>,
    {
        *self.value_written = true;
        self.encoder.emit_dict(content_cb)
    }

    /// Emit a dictionary that may have keys out of order. This will write the dict
    /// values to temporary memory, then sort them before adding them to the serialized
    /// stream
    pub fn emit_unsorted_dict<F>(self, content_cb: F) -> Result<(), Error>
    where
        F: FnOnce(&mut UnsortedDictEncoder) -> Result<(), Error>,
    {
        *self.value_written = true;
        self.encoder.emit_and_sort_dict(content_cb)
    }

    /// Emit an arbitrary list.
    ///
    /// Attention: If this method is used while canonical output is required
    /// the caller needs to ensure that the iterator has a defined order.
    pub fn emit_unchecked_list(
        self,
        iterable: impl Iterator<Item = impl ToBencode>,
    ) -> Result<(), Error> {
        self.emit_list(|e| {
            for item in iterable {
                e.emit(item)?;
            }
            Ok(())
        })
    }
}

/// Encodes a map with pre-sorted keys
pub struct SortedDictEncoder<'a> {
    encoder: &'a mut Encoder,
}

impl<'a> SortedDictEncoder<'a> {
    /// Emit a key/value pair
    pub fn emit_pair<E>(&mut self, key: &[u8], value: E) -> Result<(), Error>
    where
        E: ToBencode,
    {
        self.encoder.emit_token(Token::String(key))?;
        self.encoder.emit(value)
    }

    /// Equivalent to [`SortedDictEncoder::emit_pair()`], but forces the type of the value
    /// to be a callback
    pub fn emit_pair_with<F>(&mut self, key: &[u8], value_cb: F) -> Result<(), Error>
    where
        F: FnOnce(SingleItemEncoder) -> Result<(), Error>,
    {
        self.encoder.emit_token(Token::String(key))?;
        self.encoder.emit_with(value_cb)
    }
}

/// Helper to write a dictionary that may have keys out of order. This will buffer the
/// dict values in temporary memory, then sort them before adding them to the serialized
/// stream
pub struct UnsortedDictEncoder {
    content: BTreeMap<Vec<u8>, Vec<u8>>,
    error: Result<(), Error>,
    remaining_depth: usize,
}

impl UnsortedDictEncoder {
    pub(crate) fn new(remaining_depth: usize) -> Self {
        Self {
            content: BTreeMap::new(),
            error: Ok(()),
            remaining_depth,
        }
    }

    /// Emit a key/value pair
    pub fn emit_pair<E>(&mut self, key: &[u8], value: E) -> Result<(), Error>
    where
        E: ToBencode,
    {
        self.emit_pair_with(key, |e| value.encode(e))
    }

    /// Emit a key/value pair where the value is produced by a callback
    pub fn emit_pair_with<F>(&mut self, key: &[u8], value_cb: F) -> Result<(), Error>
    where
        F: FnOnce(SingleItemEncoder) -> Result<(), Error>,
    {
        let mut value_written = false;

        let mut encoder = Encoder::new().with_max_depth(self.remaining_depth);

        let ret = value_cb(SingleItemEncoder {
            encoder: &mut encoder,
            value_written: &mut value_written,
        });

        if ret.is_err() {
            self.error = ret.clone();
            return ret;
        }

        if !value_written {
            self.error = Err(Error::from(StructureError::InvalidState(
                "No value was emitted".to_owned(),
            )));
        } else {
            self.error = encoder.state.observe_eof().map_err(Error::from);
        }

        if self.error.is_err() {
            return self.error.clone();
        }

        let encoded_object = encoder
            .get_output()
            .expect("Any errors should have been caught by observe_eof");

        self.save_pair(key, encoded_object)
    }

    #[cfg(feature = "serde")]
    pub(crate) fn remaining_depth(&self) -> usize {
        self.remaining_depth
    }

    pub(crate) fn save_pair(
        &mut self,
        unencoded_key: &[u8],
        encoded_value: Vec<u8>,
    ) -> Result<(), Error> {
        #[cfg(not(feature = "std"))]
        use alloc::collections::btree_map::Entry;
        #[cfg(feature = "std")]
        use std::collections::btree_map::Entry;

        if self.error.is_err() {
            return self.error.clone();
        }

        let vacancy = match self.content.entry(unencoded_key.to_owned()) {
            Entry::Vacant(vacancy) => vacancy,
            Entry::Occupied(occupation) => {
                self.error = Err(Error::from(StructureError::InvalidState(format!(
                    "Duplicate key {}",
                    String::from_utf8_lossy(occupation.key())
                ))));
                return self.error.clone();
            },
        };

        vacancy.insert(encoded_value);

        Ok(())
    }

    pub(crate) fn done(self) -> Result<BTreeMap<Vec<u8>, Vec<u8>>, Error> {
        self.error?;
        Ok(self.content)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn simple_encoding_works() {
        let mut encoder = Encoder::new();
        encoder
            .emit_dict(|mut e| {
                e.emit_pair(b"bar", 25)?;
                e.emit_pair_with(b"foo", |e| {
                    e.emit_list(|e| {
                        e.emit_str("baz")?;
                        e.emit_str("qux")
                    })
                })
            })
            .expect("Encoding shouldn't fail");
        assert_eq!(
            &encoder
                .get_output()
                .expect("Complete object should have been written"),
            &b"d3:bari25e3:fool3:baz3:quxee"
        );
    }

    #[test]
    fn emit_cb_must_emit() {
        let mut encoder = Encoder::new();
        assert!(encoder.emit_with(|_| Ok(())).is_err());
    }
}
