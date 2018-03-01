//! Decodes a bencoded struct

trait Stack<T> {
    fn peek_mut(&mut self) -> Option<&mut T>;

    fn peek(&self) -> Option<&T>;

    fn replace_top(&mut self, new_value: T);
}

impl<T> Stack<T> for Vec<T> {
    fn peek_mut(&mut self) -> Option<&mut T> {
        let len = self.len();
        if len == 0 {
            None
        } else {
            Some(&mut self[len - 1])
        }
    }

    fn peek(&self) -> Option<&T> {
        let len = self.len();
        if len == 0 {
            None
        } else {
            Some(&self[len - 1])
        }
    }

    fn replace_top(&mut self, new_value: T) {
        self.peek_mut()
            .map(|top| *top = new_value)
            .expect("Shouldn't replace_top with nothing on the stack");
    }
}

/// A decoding error
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Error)]
pub enum Error {
    /// Saw the wrong type of token
    #[error(msg_embedded, no_from, non_std)]
    InvalidState(String),
    /// Keys were not sorted
    UnsortedKeys,
    /// Reached EOF in the middle of a message
    UnexpectedEof,
    /// Malformed number or unexpected character
    #[error(msg_embedded, no_from, non_std)]
    SyntaxError(String),
    /// Maximum nesting depth exceeded
    NestingTooDeep,
}

impl Error {
    fn unexpected(expected: &str, got: char, offset: usize) -> Self {
        Error::SyntaxError(format!(
            "Expected {}, got {:?} at offset {}",
            expected, got, offset
        ))
    }
}

/// A raw bencode token
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Token<'a> {
    /// The beginning of a list
    List,
    /// The beginning of a dictionary
    Dict,
    /// A byte string; may not be UTF-8
    String(&'a [u8]),
    /// A number; we explicitly *don't* parse it here, as it could be signed, unsigned, or a bignum
    Num(&'a str),
    /// The end of a list or dictionary
    End,
}

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
}

/// The state of current level of the decoder
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
enum DecodeState<'a> {
    /// An inner list. Allows any token
    Seq,
    /// Inside a map, expecting a key. Contains the last key read, so sorting can be validated
    MapKey(Option<&'a [u8]>),
    /// Inside a map, expecting a value. Contains the last key read, so sorting can be validated
    MapValue(&'a [u8]),
    /// Received an error while decoding
    Failed(Error),
}

/// A bencode decoder
///
/// This can be used to either get a stream of tokens (using the [Decoder::tokens()] method) or to
/// read a complete object at a time (using the [Decoder::next_object()]) method.
pub struct Decoder<'a> {
    source: &'a [u8],
    offset: usize,
    state: Vec<DecodeState<'a>>,
    max_depth: usize,
}

#[cfg_attr(feature = "cargo-clippy", allow(match_same_arms))]
impl<'ser> Decoder<'ser> {
    /// Create a new decoder from the given byte array
    pub fn new(buffer: &'ser [u8]) -> Self {
        Decoder {
            source: buffer,
            offset: 0,
            state: vec![],
            max_depth: 2048,
        }
    }

    /// Set the maximum nesting depth of the decoder. An unlimited-depth decoder may be
    /// created using `with_max_depth(<usize>::max_value())`, but be warned that this will likely
    /// exhaust memory if the nesting depth is too deep (even when reading raw tokens)
    pub fn with_max_depth(self, new_max_depth: usize) -> Self {
        Decoder{
            max_depth: new_max_depth,
            ..self
        }
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
            }
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
                State::Start => if c == '-' {
                    state = State::Sign;
                } else if c == '0' {
                    state = State::Zero;
                } else if c >= '1' && c <= '9' {
                    state = State::Digits;
                } else {
                    return Err(Error::unexpected("'-' or '0'..'9'", c, curpos));
                },
                State::Zero => if c == expected_terminator {
                    success = true;
                    break;
                } else {
                    return Err(Error::unexpected(
                        &format!("{:?}", expected_terminator),
                        c,
                        curpos,
                    ));
                },
                State::Sign => if c >= '1' && c <= '9' {
                    state = State::Digits;
                } else {
                    return Err(Error::unexpected("'1'..'9'", c, curpos));
                },
                State::Digits => if c >= '0' && c <= '9' {
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
                },
            }
            curpos += 1;
        }
        if success {
            let slice = &self.source[self.offset..curpos];
            self.offset = curpos + 1;
            let ival = if cfg!(debug) {
                str::from_utf8(slice).expect("We've already examined every byte in the string")
            } else {
                // Avoid a second UTF-8 check here
                unsafe { str::from_utf8_unchecked(slice) }
            };
            return Ok(ival);
        } else {
            return Err(Error::UnexpectedEof);
        }
    }

    fn latch_err<T>(&mut self, result: Result<T, Error>) -> Result<T, Error> {
        if let Err(ref err) = result {
            self.state.push(DecodeState::Failed(err.clone()))
        }
        result
    }

    fn check_error(&self) -> Result<(), Error> {
        if let Some(&DecodeState::Failed(ref error)) = self.state.peek() {
            Err(error.clone())
        } else {
            Ok(())
        }
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
            }
            tok => {
                return Err(Error::SyntaxError(format!(
                    "Invalid token starting with {:?} at offset {}",
                    tok,
                    self.offset - 1
                )))
            }
        };
        return Ok(token);
    }

    /// Read the next token. Returns Ok(Some(token)) if a token was successfully read,
    fn next_token(&mut self) -> Result<Option<Token<'ser>>, Error> {
        use self::Token::*;
        use self::DecodeState::*;

        self.check_error()?;

        let start_offset = self.offset;

        if self.offset == self.source.len() {
            if self.state.is_empty() {
                return Ok(None);
            } else {
                return self.latch_err(Err(Error::UnexpectedEof));
            }
        }

        let tok_result = self.raw_next_token();
        let tok = self.latch_err(tok_result)?;

        match (self.state.peek(), tok) {
            (None, End) => {
                self.offset = start_offset;
                return self.latch_err(Err(Error::InvalidState(
                    "End not allowed at top level".to_owned(),
                )));
            }
            (Some(&Seq), End) => {
                self.state.pop();
            }
            (Some(&MapKey(_)), End) => {
                self.state.pop();
            }
            (Some(&MapKey(None)), String(label)) => {
                self.state.replace_top(MapValue(label));
            }
            (Some(&MapKey(Some(oldlabel))), String(label)) => {
                if oldlabel >= label {
                    self.offset = start_offset;
                    return self.latch_err(Err(Error::UnsortedKeys));
                }
                self.state.replace_top(MapValue(label));
            }
            (Some(&MapKey(_)), _tok) => {
                self.offset = start_offset;
                return self.latch_err(Err(Error::InvalidState(
                    "Map keys must be strings".to_owned(),
                )));
            }
            (Some(&MapValue(label)), List) => {
                self.state.replace_top(MapKey(Some(label)));
                if self.state.len() >= self.max_depth {
                    return self.latch_err(Err(Error::NestingTooDeep))
                }
                self.state.push(Seq);
            }
            (Some(&MapValue(label)), Dict) => {
                self.state.replace_top(MapKey(Some(label)));
                if self.state.len() >= self.max_depth {
                    return self.latch_err(Err(Error::NestingTooDeep))
                }
                self.state.push(MapKey(None));
            }
            (Some(&MapValue(_)), End) => {
                return self.latch_err(Err(Error::InvalidState("Missing map value".to_owned())))
            }
            (Some(&MapValue(label)), _) => {
                self.state.replace_top(MapKey(Some(label)));
            }
            (_, List) => {
                if self.state.len() >= self.max_depth {
                    return self.latch_err(Err(Error::NestingTooDeep))
                }
                self.state.push(Seq);
            }
            (_, Dict) => {
                if self.state.len() >= self.max_depth {
                    return self.latch_err(Err(Error::NestingTooDeep))
                }
                self.state.push(MapKey(None));
            }
            (_, _) => (),
        }
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
        if self.0.check_error().is_err() {
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
pub struct DictDecoder<'obj, 'ser: 'obj> {
    decoder: &'obj mut Decoder<'ser>,
    finished: bool,
    start_point: usize,
}

/// A list read from the input stream
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

    /// Parse the next key/value pair from the dictionary
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

    /// Consume the rest of the items from the dictionary
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

    /// Get the next item from the list
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

    /// Consume the rest of the items from the list
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
    fn map_keys_must_be_strings() {
        decode_err(b"d3:fooi1ei2ei3ee", r"^Map keys must be strings$");
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
        decode_err(b"d3:fooe", r"^Missing map value$");
    }

    #[test]
    fn strings_must_have_bodies() {
        decode_err(b"3:", r"EOF");
    }

    #[test]
    fn recursion_should_be_limited() {
        use std::iter::repeat;
        let mut msg = Vec::new();
        msg.extend(repeat('l' as u8).take(4096));
        msg.extend(repeat('e' as u8).take(4096));
        decode_err(&msg, r"nesting depth");
    }

    #[test]
    fn recursion_bounds_should_be_tight() {
        let test_msg = b"lllleeee";
        assert!(Decoder::new(test_msg).with_max_depth(4).tokens().last().unwrap().is_ok());
        assert!(Decoder::new(test_msg).with_max_depth(3).tokens().last().unwrap().is_err());
    }

    #[test]
    fn dict_drop_should_consume_struct() {
        let mut decoder = Decoder::new(b"d3:fooi1e3:quxi2ee");
        drop(decoder.next_object());
        assert!(decoder.next_object().unwrap().is_none())
    }

    #[test]
    fn list_drop_should_consume_struct() {
        let mut decoder = Decoder::new(b"li1ei2ei3ee");
        drop(decoder.next_object());
        assert!(decoder.next_object().unwrap().is_none())
    }
}
