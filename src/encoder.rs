//! An encoder for bencode. Guarantees that the output string is valid bencode

use state_tracker::{StateTracker, Token};
use std::io::{self, Write};
use std::collections::BTreeMap;
use super::Error;

/// A value that can be formatted as a decimal integer
pub trait Integer {
    /// Write the value as a decimal integer
    fn write_to<W: Write>(self, w: W) -> io::Result<()>;
}

macro_rules! impl_integer {
($($type:ty)*) => {$(
impl Integer for $type {
fn write_to<W: Write>(self, mut w: W) -> io::Result<()> {
write!(w, "{}", self)
}
}
)*}
}

impl_integer!(u8 u16 u32 u64 usize i8 i16 i32 i64 isize);

impl<'a, T: Integer + Copy> Integer for &'a T {
    fn write_to<W: Write>(self, w: W) -> io::Result<()> {
        T::write_to(*self, w)
    }
}

/// The actual encoder. Unlike the decoder, this is not zero-copy, as that would
/// result in a horrible interface
#[derive(Default, Debug)]
pub struct Encoder {
    state: StateTracker<Vec<u8>>,
    output: Vec<u8>,
}

impl Encoder {
    /// Create a new encoder
    pub fn new() -> Self {
        <Self as Default>::default()
    }

    /// Emit a single token to the encoder
    fn emit_token(&mut self, token: Token) -> Result<(), Error> {
        self.state.check_error()?;
        self.state.observe_token(&token)?;
        match token {
            Token::List => self.output.push(b'l'),
            Token::Dict => self.output.push(b'd'),
            Token::String(s) => {
                // Writing to a vec can't fail
                write!(&mut self.output, "{}:", s.len()).unwrap();
                self.output.extend_from_slice(s);
            }
            Token::Num(num) => {
                // Alas, this doesn't verify that the given number is valid
                self.output.push(b'i');
                self.output.extend_from_slice(num.as_bytes());
                self.output.push(b'e');
            }
            Token::End => self.output.push(b'e'),
        }

        Ok(())
    }
}

/// Higher-level interface
impl Encoder {
    /// Emit an integer
    pub fn emit_int<T: Integer>(&mut self, value: T) -> Result<(), Error> {
        self.state.check_error()?;
        self.state.observe_token(&Token::Num(""))?; // the state tracker doesn't care about int values
        self.output.push(b'i');
        value.write_to(&mut self.output).unwrap(); // Vec can't produce an error
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

    /// Emit a dictionary where you know that the keys are already sorted
    pub fn emit_dict<F>(&mut self, content_cb: F) -> Result<(), Error>
    where
        F: FnOnce(SortedDictEncoder) -> Result<(), Error>,
    {
        self.emit_token(Token::Dict)?;
        content_cb(SortedDictEncoder { encoder: self })?;
        self.emit_token(Token::End)
    }

    /// Emit an arbitrary list
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
    pub fn emit_unsorted_dict<F>(&mut self, content_cb: F) -> Result<(), Error>
    where
        F: FnOnce(&mut UnsortedDictEncoder) -> Result<(), Error>,
    {
        // emit the dict token so that that error is reported early
        self.emit_token(Token::Dict)?;

        let mut encoder = UnsortedDictEncoder {
            content: BTreeMap::new(),
            error: Ok(()),
        };
        content_cb(&mut encoder)?;

        encoder.error?;
        for (k, v) in encoder.content {
            self.emit_bytes(&k)?;
            // We know that the output is a single object by construction
            self.state.observe_token(&Token::Num(""))?;
            self.output.extend_from_slice(&v);
        }

        self.emit_token(Token::End)
    }

    /// Return the encoded string, if all objects written are complete
    pub fn get_output(mut self) -> Result<Vec<u8>, Error> {
        self.state.observe_eof()?;
        Ok(self.output)
    }
}

/// An encoder that can only encode a single item
pub struct SingleItemEncoder<'a> {
    encoder: &'a mut Encoder,
    value_written: &'a mut bool,
}

impl<'a> SingleItemEncoder<'a> {
    /// Emit an integer
    pub fn emit_int<T: Integer>(self, value: T) -> Result<(), Error> {
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

    /// Emit a sorted dictionary. If the input dictionary is unsorted
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
        self.encoder.emit_unsorted_dict(content_cb)
    }
}

/// Encodes a map with pre-sorted keys
pub struct SortedDictEncoder<'a> {
    encoder: &'a mut Encoder,
}

impl<'a> SortedDictEncoder<'a> {
    /// Emit a key/value pair
    pub fn emit_pair<F>(&mut self, key: &[u8], value_cb: F) -> Result<(), Error>
    where
        F: FnOnce(SingleItemEncoder) -> Result<(), Error>,
    {
        use std::mem::replace;

        let mut value_written = false;

        self.encoder.emit_token(Token::String(key))?;
        let old_state = replace(&mut self.encoder.state, StateTracker::new());
        let ret = value_cb(SingleItemEncoder {
            encoder: &mut self.encoder,
            value_written: &mut value_written,
        });

        let temp_state = replace(&mut self.encoder.state, old_state);
        self.encoder.state.latch_err(temp_state.check_error())?;
        if !value_written {
            return self.encoder
                .state
                .latch_err(Err(Error::InvalidState("No value was emitted".to_owned())));
        }
        ret
    }
}

/// Helper to write a dictionary that may have keys out of order. This will buffer the
/// dict values in temporary memory, then sort them before adding them to the serialized
/// stream
pub struct UnsortedDictEncoder {
    content: BTreeMap<Vec<u8>, Vec<u8>>,
    error: Result<(), Error>,
}

impl UnsortedDictEncoder {
    /// Emit a key/value pair
    pub fn emit_pair<F>(&mut self, key: &[u8], value_cb: F) -> Result<(), Error>
    where
        F: FnOnce(SingleItemEncoder) -> Result<(), Error>,
    {
        if self.error.is_err() {
            return self.error.clone();
        }

        let mut value_written = false;

        let mut encoder = Encoder::new();

        let ret = value_cb(SingleItemEncoder {
            encoder: &mut encoder,
            value_written: &mut value_written,
        });

        if !value_written {
            self.error = Err(Error::InvalidState("No value was emitted".to_owned()));
        } else {
            self.error = encoder.state.observe_eof();
        }

        if self.content
            .insert(key.to_owned(), encoder.get_output()?)
            .is_some()
        {
            self.error = Err(Error::InvalidState(format!(
                "Duplicate key {}",
                String::from_utf8_lossy(key)
            )))
        }

        if self.error.is_err() {
            return self.error.clone();
        }

        ret
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
                e.emit_pair(b"bar", |e| e.emit_int(25))?;
                e.emit_pair(b"foo", |e| {
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
}
