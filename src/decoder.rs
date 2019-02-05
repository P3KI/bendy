//! Decodes a bencoded struct
//!
//! # Basic decoding
//! For any decoding process, first we need to create a decoder:
//!
//! ```
//! # use bendy::decoder::{Decoder,Object};
//! # use bendy::Error;
//! #
//! # let buf: &[u8] = b"d3:fooi1ee";
//! let mut decoder = Decoder::new(buf);
//! ```
//!
//! Decoders have a depth limit to prevent resource exhaustion from hostile inputs. By default, it's
//! set high enough for most structures that you'd encounter when prototyping, but for production
//! use, not only may it not be enough, but the higher the depth limit, the more stack space an
//! attacker can cause your program to use, so we recommend setting the bounds tightly:
//!
//! ```
//! # use bendy::decoder::{Decoder,Object};
//! # use bendy::Error;
//! #
//! # let buf: &[u8] = b"d3:fooi1ee";
//! # let mut decoder = Decoder::new(buf);
//! #
//! decoder = decoder.with_max_depth(3);
//! ```
//!
//! Atoms (integers and strings) have depth zero, and lists and dicts have a depth equal to the
//! depth of their deepest member plus one. As an special case, an empty list or dict has depth 1.
//!
//! Now, you can start reading objects:
//!
//! ```
//! # use bendy::decoder::{Decoder,Object};
//! # use bendy::Error;
//! #
//! # fn decode_list(_: bendy::decoder::ListDecoder) {}
//! # fn decode_dict(_: bendy::decoder::DictDecoder) {}
//! #
//! # let buf: &[u8] = b"d3:fooi1ee";
//! # let mut decoder = Decoder::new(buf);
//! #
//! match decoder.next_object().unwrap() {
//!     None => (), // EOF
//!     Some(Object::List(d)) => decode_list(d),
//!     Some(Object::Dict(d)) => decode_dict(d),
//!     Some(Object::Integer(s)) => (), // integer, as a string
//!     Some(Object::Bytes(b)) => (), // A raw bytestring
//! };
//! ```
//!
//! # Error handling
//!
//! Once an error is encountered, the decoder won't try to muddle through it; instead, every future
//! call to the decoder will return the same error. This behaviour can be used to check the syntax
//! of an input object without fully decoding it:
//!
//! ```
//! # use bendy::decoder::Decoder;
//! #
//! fn syntax_check(buf: &[u8]) -> bool {
//!     let mut decoder = Decoder::new(buf);
//!     decoder.next_object().ok(); // ignore the return value of this
//!     return decoder.next_object().is_ok();
//! }
//! ```

use crate::{state_tracker::StateTracker, token::Token};

use super::Error;

/// An object read from a decoder
pub enum Object<'obj, 'ser: 'obj> {
    /// A list of arbitrary objects
    List(ListDecoder<'obj, 'ser>),
    /// A map of string-valued keys to arbitrary objects
    Dict(DictDecoder<'obj, 'ser>),
    /// An unparsed integer
    Integer(&'ser str),
    /// A byte string
    Bytes(&'ser [u8]),
}

impl<'obj, 'ser: 'obj> Object<'obj, 'ser> {
    fn into_token(self) -> Token<'ser> {
        match self {
            Object::List(_) => Token::List,
            Object::Dict(_) => Token::Dict,
            Object::Bytes(bytes) => Token::String(bytes),
            Object::Integer(num) => Token::Num(num),
        }
    }

    /// Try to treat the object as a byte string, mapping [`Object::Bytes(v)`] into
    /// [`Ok(v)`] and any other variant to [`Err(error)`].
    ///
    /// Arguments passed to `bytes_or_err` are eagerly evaluated; if you are passing the
    /// result of a function call, it is recommended to use [`bytes_or_else_err`], which is
    /// lazily evaluated.
    ///
    /// [`Object::Bytes(v)`]: self::Object::Bytes
    /// [`Ok(v)`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Ok
    /// [`Err(error)`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Err
    /// [`bytes_or_else_err`]: self::Object::bytes_or_else_err
    ///
    /// # Examples
    ///
    /// ```
    /// use bendy::decoder::Object;
    ///
    /// let x = Object::Bytes(b"foo");
    /// assert_eq!(Ok(&b"foo"[..]), x.bytes_or_err(0));
    ///
    /// let x = Object::Integer("foo");
    /// assert_eq!(Err(0), x.bytes_or_err(0));
    /// ```
    pub fn bytes_or_err<ErrorT>(self, error: ErrorT) -> Result<&'ser [u8], ErrorT> {
        match self {
            Object::Bytes(content) => Ok(content),
            _ => Err(error),
        }
    }

    /// Try to treat the object as a byte string, mapping [`Object::Bytes(v)`] into
    /// [`Ok(v)`] and any other variant to [`Err(error())`].
    ///
    /// [`Object::Bytes(v)`]: self::Object::Bytes
    /// [`Ok(v)`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Ok
    /// [`Err(error())`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Err
    ///
    /// # Examples
    ///
    /// ```
    /// use bendy::decoder::Object;
    ///
    /// let x = Object::Bytes(b"foo");
    /// assert_eq!(Ok(&b"foo"[..]), x.bytes_or_else_err(|| 0));
    ///
    /// let x = Object::Integer("foo");
    /// assert_eq!(Err(0), x.bytes_or_else_err(|| 0));
    /// ```
    pub fn bytes_or_else_err<ErrorT, FunctionT>(
        self,
        error: FunctionT,
    ) -> Result<&'ser [u8], ErrorT>
    where
        FunctionT: FnOnce() -> ErrorT,
    {
        match self {
            Object::Bytes(content) => Ok(content),
            _ => Err(error()),
        }
    }

    /// Try to treat the object as an integer and return the internal string representation,
    /// mapping [`Object::Integer(v)`] into [`Ok(v)`] and any other variant to [`Err(error)`].
    ///
    /// Arguments passed to `integer_str_or_err` are eagerly evaluated; if you are passing the
    /// result of a function call, it is recommended to use [`integer_str_or_else_err`], which is
    /// lazily evaluated.
    ///
    /// [`Object::Integer(v)`]: self::Object::Integer
    /// [`Ok(v)`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Ok
    /// [`Err(error)`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Err
    /// [`integer_str_or_else_err`]: self::Object::integer_str_or_else_err
    ///
    /// # Examples
    ///
    /// ```
    /// use bendy::decoder::Object;
    ///
    /// let x = Object::Integer("123");
    /// assert_eq!(Ok(&"123"[..]), x.integer_str_or_err(-1));
    ///
    /// let x = Object::Bytes(b"foo");
    /// assert_eq!(Err(-1), x.integer_str_or_err(-1));
    /// ```
    pub fn integer_str_or_err<ErrorT>(self, error: ErrorT) -> Result<&'ser str, ErrorT> {
        match self {
            Object::Integer(content) => Ok(content),
            _ => Err(error),
        }
    }

    /// Try to treat the object as an integer and return the internal string representation,
    /// mapping [`Object::Integer(v)`] into [`Ok(v)`] and any other variant to [`Err(error())`].
    ///
    /// [`Object::Integer(v)`]: self::Object::Integer
    /// [`Ok(v)`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Ok
    /// [`Err(error())`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Err
    ///
    /// # Examples
    ///
    /// ```
    /// use bendy::decoder::Object;
    ///
    /// let x = Object::Integer("123");
    /// assert_eq!(Ok(&"123"[..]), x.integer_str_or_else_err(|| -1));
    ///
    /// let x = Object::Bytes(b"foo");
    /// assert_eq!(Err(-1), x.integer_str_or_else_err(|| -1));
    /// ```
    pub fn integer_str_or_else_err<ErrorT, FunctionT>(
        self,
        error: FunctionT,
    ) -> Result<&'ser str, ErrorT>
    where
        FunctionT: FnOnce() -> ErrorT,
    {
        match self {
            Object::Integer(content) => Ok(content),
            _ => Err(error()),
        }
    }

    /// Try to treat the object as a list and return the internal list content decoder,
    /// mapping [`Object::List(v)`] into [`Ok(v)`] and any other variant to [`Err(error)`].
    ///
    /// Arguments passed to `list_or_err` are eagerly evaluated; if you are passing the
    /// result of a function call, it is recommended to use [`list_or_else_err`], which is
    /// lazily evaluated.
    ///
    /// [`Object::List(v)`]: self::Object::List
    /// [`Ok(v)`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Ok
    /// [`Err(error())`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Err
    /// [`list_or_else_err`]: self::Object::list_or_else_err
    ///
    /// # Examples
    ///
    /// ```
    /// use bendy::decoder::{Decoder, Object};
    ///
    /// let mut list_decoder = Decoder::new(b"le");
    /// let x = list_decoder.next_object().unwrap().unwrap();
    ///
    /// assert!(x.list_or_err(0).is_ok());
    ///
    /// let x = Object::Bytes(b"foo");
    /// assert_eq!(0, x.list_or_err(0).unwrap_err());
    /// ```
    pub fn list_or_err<ErrorT>(self, error: ErrorT) -> Result<ListDecoder<'obj, 'ser>, ErrorT> {
        match self {
            Object::List(content) => Ok(content),
            _ => Err(error),
        }
    }

    /// Try to treat the object as a list and return the internal list content decoder,
    /// mapping [`Object::List(v)`] into [`Ok(v)`] and any other variant to [`Err(error())`].
    ///
    /// [`Object::List(v)`]: self::Object::List
    /// [`Ok(v)`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Ok
    /// [`Err(error())`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Err
    ///
    /// # Examples
    ///
    /// ```
    /// use bendy::decoder::{Decoder, Object};
    ///
    /// let mut list_decoder = Decoder::new(b"le");
    /// let x = list_decoder.next_object().unwrap().unwrap();
    ///
    /// assert!(x.list_or_else_err(|| 0).is_ok());
    ///
    /// let x = Object::Bytes(b"foo");
    /// assert_eq!(0, x.list_or_else_err(|| 0).unwrap_err());
    /// ```
    pub fn list_or_else_err<ErrorT, FunctionT>(
        self,
        error: FunctionT,
    ) -> Result<ListDecoder<'obj, 'ser>, ErrorT>
    where
        FunctionT: FnOnce() -> ErrorT,
    {
        match self {
            Object::List(content) => Ok(content),
            _ => Err(error()),
        }
    }

    /// Try to treat the object as a dictionary and return the internal dictionary content
    /// decoder, mapping [`Object::Dict(v)`] into [`Ok(v)`] and any other variant to
    /// [`Err(error)`].
    ///
    /// Arguments passed to `dictionary_or_err` are eagerly evaluated; if you are passing the
    /// result of a function call, it is recommended to use [`dictionary_or_else_err`], which is
    /// lazily evaluated.
    ///
    /// [`Object::Dict(v)`]: self::Object::Dict
    /// [`Ok(v)`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Ok
    /// [`Err(error)`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Err
    /// [`dictionary_or_else_err`]: self::Object::dictionary_or_else_err
    ///
    /// # Examples
    ///
    /// ```
    /// use bendy::decoder::{Decoder, Object};
    ///
    /// let mut dict_decoder = Decoder::new(b"de");
    /// let x = dict_decoder.next_object().unwrap().unwrap();
    ///
    /// assert!(x.dictionary_or_err(0).is_ok());
    ///
    /// let x = Object::Bytes(b"foo");
    /// assert_eq!(0, x.dictionary_or_err(0).unwrap_err());
    /// ```
    pub fn dictionary_or_err<ErrorT>(
        self,
        error: ErrorT,
    ) -> Result<DictDecoder<'obj, 'ser>, ErrorT> {
        match self {
            Object::Dict(content) => Ok(content),
            _ => Err(error),
        }
    }

    /// Try to treat the object as a dictionary and return the internal dictionary content
    /// decoder, mapping [`Object::Dict(v)`] into [`Ok(v)`] and any other variant to
    /// [`Err(error())`].
    ///
    /// [`Object::Dict(v)`]: self::Object::Dict
    /// [`Ok(v)`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Ok
    /// [`Err(error())`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Err
    ///
    /// # Examples
    ///
    /// ```
    /// use bendy::decoder::{Decoder, Object};
    ///
    /// let mut dict_decoder = Decoder::new(b"de");
    /// let x = dict_decoder.next_object().unwrap().unwrap();
    ///
    /// assert!(x.dictionary_or_else_err(|| 0).is_ok());
    ///
    /// let x = Object::Bytes(b"foo");
    /// assert_eq!(0, x.dictionary_or_else_err(|| 0).unwrap_err());
    /// ```
    pub fn dictionary_or_else_err<ErrorT, FunctionT>(
        self,
        error: FunctionT,
    ) -> Result<DictDecoder<'obj, 'ser>, ErrorT>
    where
        FunctionT: FnOnce() -> ErrorT,
    {
        match self {
            Object::Dict(content) => Ok(content),
            _ => Err(error()),
        }
    }
}

/// A bencode decoder
///
/// This can be used to either get a stream of tokens (using the [`Decoder::tokens()`] method) or to
/// read a complete object at a time (using the [`Decoder::next_object()`]) method.
#[derive(Debug)]
pub struct Decoder<'a> {
    source: &'a [u8],
    offset: usize,
    state: StateTracker<&'a [u8]>,
}

impl<'ser> Decoder<'ser> {
    /// Create a new decoder from the given byte array
    pub fn new(buffer: &'ser [u8]) -> Self {
        Decoder {
            source: buffer,
            offset: 0,
            state: StateTracker::new(),
        }
    }

    /// Set the maximum nesting depth of the decoder. An unlimited-depth decoder may be
    /// created using `with_max_depth(<usize>::max_value())`, but be warned that this will likely
    /// exhaust memory if the nesting depth is too deep (even when reading raw tokens)
    pub fn with_max_depth(mut self, new_max_depth: usize) -> Self {
        self.state.set_max_depth(new_max_depth);
        self
    }

    fn take_byte(&mut self) -> Option<u8> {
        if self.offset < self.source.len() {
            let ret = Some(self.source[self.offset]);
            self.offset += 1;
            ret
        } else {
            None
        }
    }

    fn take_chunk(&mut self, count: usize) -> Option<&'ser [u8]> {
        match self.offset.checked_add(count) {
            Some(end_pos) if end_pos <= self.source.len() => {
                let ret = &self.source[self.offset..end_pos];
                self.offset = end_pos;
                Some(ret)
            },
            _ => None,
        }
    }

    fn take_int(&mut self, expected_terminator: char) -> Result<&'ser str, Error> {
        use std::str;
        enum State {
            Start,
            Sign,
            Zero,
            Digits,
        }

        let mut curpos = self.offset;
        let mut state = State::Start;

        let mut success = false;
        while curpos < self.source.len() {
            let c = self.source[curpos] as char;
            match state {
                State::Start => {
                    if c == '-' {
                        state = State::Sign;
                    } else if c == '0' {
                        state = State::Zero;
                    } else if c >= '1' && c <= '9' {
                        state = State::Digits;
                    } else {
                        return Err(Error::unexpected("'-' or '0'..'9'", c, curpos));
                    }
                },
                State::Zero => {
                    if c == expected_terminator {
                        success = true;
                        break;
                    } else {
                        return Err(Error::unexpected(
                            &format!("{:?}", expected_terminator),
                            c,
                            curpos,
                        ));
                    }
                },
                State::Sign => {
                    if c >= '1' && c <= '9' {
                        state = State::Digits;
                    } else {
                        return Err(Error::unexpected("'1'..'9'", c, curpos));
                    }
                },
                State::Digits => {
                    if c >= '0' && c <= '9' {
                        // do nothing, this is ok
                    } else if c == expected_terminator {
                        success = true;
                        break;
                    } else {
                        return Err(Error::unexpected(
                            &format!("{:?} or '0'..'9'", expected_terminator),
                            c,
                            curpos,
                        ));
                    }
                },
            }
            curpos += 1;
        }

        if !success {
            return Err(Error::UnexpectedEof);
        }

        let slice = &self.source[self.offset..curpos];
        self.offset = curpos + 1;
        let ival = if cfg!(debug) {
            str::from_utf8(slice).expect("We've already examined every byte in the string")
        } else {
            // Avoid a second UTF-8 check here
            unsafe { str::from_utf8_unchecked(slice) }
        };

        Ok(ival)
    }

    fn raw_next_token(&mut self) -> Result<Token<'ser>, Error> {
        let token = match self.take_byte().ok_or(Error::UnexpectedEof)? as char {
            'e' => Token::End,
            'l' => Token::List,
            'd' => Token::Dict,
            'i' => Token::Num(self.take_int('e')?),
            c if c >= '0' && c <= '9' => {
                self.offset -= 1;

                let curpos = self.offset;
                let ival = self.take_int(':')?;
                let len = usize::from_str_radix(ival, 10).map_err(|_| {
                    Error::SyntaxError(format!("Invalid integer at offset {}", curpos))
                })?;
                Token::String(self.take_chunk(len).ok_or(Error::UnexpectedEof)?)
            },
            tok => {
                return Err(Error::SyntaxError(format!(
                    "Invalid token starting with {:?} at offset {}",
                    tok,
                    self.offset - 1
                )));
            },
        };

        Ok(token)
    }

    /// Read the next token. Returns Ok(Some(token)) if a token was successfully read,
    fn next_token(&mut self) -> Result<Option<Token<'ser>>, Error> {
        self.state.check_error()?;

        if self.offset == self.source.len() {
            self.state.observe_eof()?;
            return Ok(None);
        }

        let tok_result = self.raw_next_token();
        let tok = self.state.latch_err(tok_result)?;

        self.state.observe_token(&tok)?;
        Ok(Some(tok))
    }

    /// Iterate over the tokens in the input stream. This guarantees that the resulting stream
    /// of tokens constitutes a valid bencoded structure.
    pub fn tokens(self) -> Tokens<'ser> {
        Tokens(self)
    }
}

/// Iterator over the tokens in the input stream. This guarantees that the resulting stream
/// of tokens constitutes a valid bencoded structure.
pub struct Tokens<'a>(Decoder<'a>);

impl<'a> Iterator for Tokens<'a> {
    type Item = Result<Token<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        // Only report an error once
        if self.0.state.check_error().is_err() {
            return None;
        }
        match self.0.next_token() {
            Ok(Some(token)) => Some(Ok(token)),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        }
    }
}

// High level interface

impl<'ser> Decoder<'ser> {
    /// Read the next object from the encoded stream
    ///
    /// If the beginning of an object was successfully read, returns `Ok(Some(object))`.
    /// At the end of the input stream, this will return `Ok(None)`; otherwise, returns
    /// `Err(some_error)`.
    ///
    /// Note that complex objects (lists and dicts) are not fully validated before being
    /// returned from this method, so you may still get an error while decoding the contents
    /// of the object
    pub fn next_object<'obj>(&'obj mut self) -> Result<Option<Object<'obj, 'ser>>, Error> {
        use self::Token::*;
        Ok(match self.next_token()? {
            None | Some(End) => None,
            Some(List) => Some(Object::List(ListDecoder::new(self))),
            Some(Dict) => Some(Object::Dict(DictDecoder::new(self))),
            Some(String(s)) => Some(Object::Bytes(s)),
            Some(Num(s)) => Some(Object::Integer(s)),
        })
    }
}

/// A dictionary read from the input stream
#[derive(Debug)]
pub struct DictDecoder<'obj, 'ser: 'obj> {
    decoder: &'obj mut Decoder<'ser>,
    finished: bool,
    start_point: usize,
}

/// A list read from the input stream
#[derive(Debug)]
pub struct ListDecoder<'obj, 'ser: 'obj> {
    decoder: &'obj mut Decoder<'ser>,
    finished: bool,
    start_point: usize,
}

impl<'obj, 'ser: 'obj> DictDecoder<'obj, 'ser> {
    fn new(decoder: &'obj mut Decoder<'ser>) -> Self {
        let offset = decoder.offset - 1;
        DictDecoder {
            decoder,
            finished: false,
            start_point: offset,
        }
    }

    /// Parse the next key/value pair from the dictionary. Returns `Ok(None)`
    /// at the end of the dictionary
    pub fn next_pair<'item>(
        &'item mut self,
    ) -> Result<Option<(&'ser [u8], Object<'item, 'ser>)>, Error> {
        if self.finished {
            return Ok(None);
        }

        // We convert to a token to release the mut ref to decoder
        let key = self.decoder.next_object()?.map(Object::into_token);

        if let Some(Token::String(k)) = key {
            // This unwrap should be safe because None would produce an error here
            let v = self.decoder.next_object()?.unwrap();
            Ok(Some((k, v)))
        } else {
            // We can't have gotten anything but a string, as anything else would be
            // a state error
            self.finished = true;
            Ok(None)
        }
    }

    /// Consume (and validate the structure of) the rest of the items from the
    /// dictionary. This method should be used to check for encoding errors if
    /// [`DictDecoder::next_pair`] is not called until it returns `Ok(None)`.
    pub fn consume_all(&mut self) -> Result<(), Error> {
        while let Some(_) = self.next_pair()? {
            // just drop the items
        }
        Ok(())
    }

    /// Get the raw bytes that made up this dictionary
    pub fn into_raw(mut self) -> Result<&'ser [u8], Error> {
        self.consume_all()?;
        Ok(&self.decoder.source[self.start_point..self.decoder.offset])
    }
}

impl<'obj, 'ser: 'obj> Drop for DictDecoder<'obj, 'ser> {
    fn drop(&mut self) {
        // we don't care about errors in drop; they'll be reported again in the parent
        self.consume_all().ok();
    }
}

impl<'obj, 'ser: 'obj> ListDecoder<'obj, 'ser> {
    fn new(decoder: &'obj mut Decoder<'ser>) -> Self {
        let offset = decoder.offset - 1;
        ListDecoder {
            decoder,
            finished: false,
            start_point: offset,
        }
    }

    /// Get the next item from the list. Returns `Ok(None)` at the end of the list
    pub fn next_object<'item>(&'item mut self) -> Result<Option<Object<'item, 'ser>>, Error> {
        if self.finished {
            return Ok(None);
        }

        let item = self.decoder.next_object()?;
        if item.is_none() {
            self.finished = true;
        }

        Ok(item)
    }

    /// Consume (and validate the structure of) the rest of the items from the
    /// list. This method should be used to check for encoding errors if
    /// [`ListDecoder::next_object`] is not called until it returns [`Ok(())`].
    ///
    /// [`Ok(())`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Ok
    pub fn consume_all(&mut self) -> Result<(), Error> {
        while let Some(_) = self.next_object()? {
            // just drop the items
        }
        Ok(())
    }

    /// Get the raw bytes that made up this list
    pub fn into_raw(mut self) -> Result<&'ser [u8], Error> {
        self.consume_all()?;
        Ok(&self.decoder.source[self.start_point..self.decoder.offset])
    }
}

impl<'obj, 'ser: 'obj> Drop for ListDecoder<'obj, 'ser> {
    fn drop(&mut self) {
        // we don't care about errors in drop; they'll be reported again in the parent
        self.consume_all().ok();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use regex;

    static SIMPLE_MSG: &'static [u8] = b"d3:bari1e3:fooli2ei3eee";

    fn decode_tokens(msg: &[u8]) -> Vec<Token> {
        let tokens: Vec<Result<Token, Error>> = Decoder::new(msg).tokens().collect();
        if tokens.iter().all(Result::is_ok) {
            tokens.into_iter().map(Result::unwrap).collect()
        } else {
            panic!(
                "Unexpected tokenization error. Received tokens: {:?}",
                tokens
            );
        }
    }

    fn decode_err(msg: &[u8], err_regex: &str) {
        let mut tokens: Vec<Result<Token, self::Error>> = Decoder::new(msg).tokens().collect();
        if tokens.iter().all(Result::is_ok) {
            panic!("Unexpected parse success: {:?}", tokens);
        } else {
            let err = format!("{}", tokens.pop().unwrap().err().unwrap());
            let err_regex = regex::Regex::new(err_regex).expect("Test regexes should be valid");
            if !err_regex.is_match(&err) {
                panic!("Unexpected error: {}", err);
            }
        }
    }

    #[test]
    fn simple_bdecode_tokenization() {
        use super::Token::*;
        let tokens: Vec<_> = decode_tokens(SIMPLE_MSG);
        assert_eq!(
            tokens,
            vec![
                Dict,
                String(&b"bar"[..]),
                Num(&"1"[..]),
                String(&b"foo"[..]),
                List,
                Num(&"2"[..]),
                Num(&"3"[..]),
                End,
                End,
            ]
        );
    }

    #[test]
    fn short_dict_should_fail() {
        decode_err(b"d", r"EOF");
    }

    #[test]
    fn short_list_should_fail() {
        decode_err(b"l", r"EOF");
    }

    #[test]
    fn short_int_should_fail() {
        decode_err(b"i12", r"EOF");
    }

    #[test]
    fn negative_numbers_and_zero_should_parse() {
        use super::Token::*;
        let tokens: Vec<_> = decode_tokens(b"i0ei-1e");
        assert_eq!(tokens, vec![Num(&"0"), Num(&"-1")],);
    }

    #[test]
    fn negative_zero_is_illegal() {
        decode_err(b"i-0e", "got '0'");
    }

    #[test]
    fn leading_zeros_are_illegal() {
        decode_err(b"i01e", "got '1'");
        decode_err(b"i-01e", "got '0'");
    }

    #[test]
    fn map_keys_must_be_strings() {
        decode_err(b"d3:fooi1ei2ei3ee", r"Map keys must be strings");
    }

    #[test]
    fn map_keys_must_ascend() {
        decode_err(b"d3:fooi1e3:bari1ee", r"Keys were not sorted");
    }

    #[test]
    fn map_keys_must_be_unique() {
        decode_err(b"d3:fooi1e3:fooi1ee", r"Keys were not sorted");
    }

    #[test]
    fn map_keys_must_have_values() {
        decode_err(b"d3:fooe", r"Missing map value");
    }

    #[test]
    fn strings_must_have_bodies() {
        decode_err(b"3:", r"EOF");
    }

    #[test]
    fn ints_must_have_bodies() {
        decode_err(b"ie", r"Expected.*got 'e'");
    }

    #[test]
    fn recursion_should_be_limited() {
        use std::iter::repeat;
        let mut msg = Vec::new();
        msg.extend(repeat(b'l').take(4096));
        msg.extend(repeat(b'e').take(4096));
        decode_err(&msg, r"nesting depth");
    }

    #[test]
    fn recursion_bounds_should_be_tight() {
        let test_msg = b"lllleeee";
        assert!(Decoder::new(test_msg)
            .with_max_depth(4)
            .tokens()
            .last()
            .unwrap()
            .is_ok());
        assert!(Decoder::new(test_msg)
            .with_max_depth(3)
            .tokens()
            .last()
            .unwrap()
            .is_err());
    }

    #[test]
    fn dict_drop_should_consume_struct() {
        let mut decoder = Decoder::new(b"d3:fooi1e3:quxi2eei1000e");
        drop(decoder.next_object());
        assert_eq!(decoder.tokens().next(), Some(Ok(Token::Num("1000"))));
    }

    #[test]
    fn list_drop_should_consume_struct() {
        let mut decoder = Decoder::new(b"li1ei2ei3eei1000e");
        drop(decoder.next_object());
        assert_eq!(decoder.tokens().next(), Some(Ok(Token::Num("1000"))));
    }

    #[test]
    fn bytes_or_should_work_on_bytes() {
        assert_eq!(Ok(&b"foo"[..]), Object::Bytes(b"foo").bytes_or_err(0));
    }

    #[test]
    fn bytes_or_should_not_work_on_other_types() {
        assert_eq!(Err(0), Object::Integer("123").bytes_or_err(0));
        let mut list_decoder = Decoder::new(b"le");
        assert_eq!(
            Err(0),
            list_decoder.next_object().unwrap().unwrap().bytes_or_err(0)
        );
        let mut dict_decoder = Decoder::new(b"de");
        assert_eq!(
            Err(0),
            dict_decoder.next_object().unwrap().unwrap().bytes_or_err(0)
        );
    }

    #[test]
    fn bytes_or_else_should_work_on_bytes() {
        assert_eq!(
            Ok(&b"foo"[..]),
            Object::Bytes(b"foo").bytes_or_else_err(|| 0)
        );
    }

    #[test]
    fn bytes_or_else_should_not_work_on_other_types() {
        assert_eq!(Err(0), Object::Integer("123").bytes_or_else_err(|| 0));
        let mut list_decoder = Decoder::new(b"le");
        assert_eq!(
            Err(0),
            list_decoder
                .next_object()
                .unwrap()
                .unwrap()
                .bytes_or_else_err(|| 0)
        );
        let mut dict_decoder = Decoder::new(b"de");
        assert_eq!(
            Err(0),
            dict_decoder
                .next_object()
                .unwrap()
                .unwrap()
                .bytes_or_else_err(|| 0)
        );
    }

    #[test]
    fn integer_str_or_should_work_on_int() {
        assert_eq!(
            Ok(&"123"[..]),
            Object::Integer("123").integer_str_or_err(-1)
        );
    }

    #[test]
    fn integer_str_or_should_not_work_on_other_types() {
        assert_eq!(Err(-1), Object::Bytes(b"foo").integer_str_or_err(-1));
        let mut list_decoder = Decoder::new(b"le");
        assert_eq!(
            Err(-1),
            list_decoder
                .next_object()
                .unwrap()
                .unwrap()
                .integer_str_or_err(-1)
        );
        let mut dict_decoder = Decoder::new(b"de");
        assert_eq!(
            Err(-1),
            dict_decoder
                .next_object()
                .unwrap()
                .unwrap()
                .integer_str_or_err(-1)
        );
    }

    #[test]
    fn integer_str_or_else_should_work_on_int() {
        assert_eq!(
            Ok(&"123"[..]),
            Object::Integer("123").integer_str_or_else_err(|| -1)
        );
    }

    #[test]
    fn integer_str_or_else_should_not_work_on_other_types() {
        assert_eq!(
            Err(-1),
            Object::Bytes(b"foo").integer_str_or_else_err(|| -1)
        );
        let mut list_decoder = Decoder::new(b"le");
        assert_eq!(
            Err(-1),
            list_decoder
                .next_object()
                .unwrap()
                .unwrap()
                .integer_str_or_else_err(|| -1)
        );
        let mut dict_decoder = Decoder::new(b"de");
        assert_eq!(
            Err(-1),
            dict_decoder
                .next_object()
                .unwrap()
                .unwrap()
                .integer_str_or_else_err(|| -1)
        );
    }

    #[test]
    fn list_or_should_work_on_list() {
        let mut list_decoder = Decoder::new(b"le");
        assert!(list_decoder
            .next_object()
            .unwrap()
            .unwrap()
            .list_or_err(0)
            .is_ok());
    }
    #[test]
    fn list_or_should_not_work_on_other_types() {
        assert_eq!(0, Object::Bytes(b"foo").list_or_err(0).unwrap_err());
        assert_eq!(0, Object::Integer("foo").list_or_err(0).unwrap_err());

        let mut dict_decoder = Decoder::new(b"de");
        assert_eq!(
            0,
            dict_decoder
                .next_object()
                .unwrap()
                .unwrap()
                .list_or_err(0)
                .unwrap_err()
        );
    }

    #[test]
    fn list_or_else_should_work_on_list() {
        let mut list_decoder = Decoder::new(b"le");
        assert!(list_decoder
            .next_object()
            .unwrap()
            .unwrap()
            .list_or_else_err(|| 0)
            .is_ok());
    }
    #[test]
    fn list_or_else_should_not_work_on_other_types() {
        assert_eq!(0, Object::Bytes(b"foo").list_or_else_err(|| 0).unwrap_err());
        assert_eq!(
            0,
            Object::Integer("foo").list_or_else_err(|| 0).unwrap_err()
        );

        let mut dict_decoder = Decoder::new(b"de");
        assert_eq!(
            0,
            dict_decoder
                .next_object()
                .unwrap()
                .unwrap()
                .list_or_else_err(|| 0)
                .unwrap_err()
        );
    }

    #[test]
    fn dictionary_or_should_work_on_dict() {
        let mut dict_decoder = Decoder::new(b"de");
        assert!(dict_decoder
            .next_object()
            .unwrap()
            .unwrap()
            .dictionary_or_err(0)
            .is_ok());
    }

    #[test]
    fn dictionary_or_should_not_work_on_other_types() {
        assert_eq!(0, Object::Bytes(b"foo").dictionary_or_err(0).unwrap_err());
        assert_eq!(0, Object::Integer("foo").dictionary_or_err(0).unwrap_err());

        let mut list_decoder = Decoder::new(b"le");
        assert_eq!(
            0,
            list_decoder
                .next_object()
                .unwrap()
                .unwrap()
                .dictionary_or_err(0)
                .unwrap_err()
        );
    }

    #[test]
    fn dictionary_or_else_should_work_on_dict() {
        let mut dict_decoder = Decoder::new(b"de");
        assert!(dict_decoder
            .next_object()
            .unwrap()
            .unwrap()
            .dictionary_or_else_err(|| 0)
            .is_ok());
    }

    #[test]
    fn dictionary_or_else_should_not_work_on_other_types() {
        assert_eq!(
            0,
            Object::Bytes(b"foo")
                .dictionary_or_else_err(|| 0)
                .unwrap_err()
        );
        assert_eq!(
            0,
            Object::Integer("foo")
                .dictionary_or_else_err(|| 0)
                .unwrap_err()
        );

        let mut list_decoder = Decoder::new(b"le");
        assert_eq!(
            0,
            list_decoder
                .next_object()
                .unwrap()
                .unwrap()
                .dictionary_or_else_err(|| 0)
                .unwrap_err()
        );
    }
}
