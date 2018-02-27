//! Decodes a bencoded struct
trait Stack<T> {
    fn peek_mut(&mut self) -> Option<&mut T>;

    fn peek(&self) -> Option<&T>;

    fn replace_top(&mut self, new_value: T) {
        self.peek_mut()
            .map(|top| *top = new_value)
            .expect("Shouldn't replace_top with nothing on the stack");
    }
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
}

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
}

impl Error {
    fn unexpected(expected: &str, got: char, offset: usize) -> Self {
        Error::SyntaxError(format!(
            "Expected {}, got {:?} at offset {}",
            expected, got, offset
        ))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Token<'a> {
    List,
    Dict,
    String(&'a [u8]),
    /// A number; we explicitly *don't* parse it here, as it could be signed, unsigned, or a bignum
    Num(&'a str),
    End,
}

/// An object read from a decoder
pub enum Object<'obj, 'ser: 'obj> {
    List(ListDecoder<'obj, 'ser>),
    Dict(DictDecoder<'obj, 'ser>),
    Integer(&'ser str),
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

///
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

pub struct Decoder<'a> {
    source: &'a [u8],
    offset: usize,
    state: Vec<DecodeState<'a>>,
}

impl<'ser> Decoder<'ser> {
    pub fn new(buffer: &'ser [u8]) -> Self {
        Decoder {
            source: buffer,
            offset: 0,
            state: vec![],
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
            Some(end_pos) if end_pos < self.source.len() => {
                let ret = &self.source[self.offset..end_pos];
                self.offset = end_pos;
                Some(ret)
            }
            _ => None,
        }
    }

    fn take_int(&mut self, expected_end: char) -> Result<&'ser str, Error> {
        use std::str;
        enum State {
            Start,
            Sign,
            Zero,
            Digits,
        }

        let mut endpos = self.offset;
        let mut state = State::Start;

        let mut success = false;
        while endpos < self.source.len() {
            let c = self.source[endpos] as char;
            endpos += 1;
            match state {
                State::Start => if c == '-' {
                    state = State::Sign;
                } else if c == '0' {
                    state = State::Zero;
                } else if c >= '1' && c <= '9' {
                    state = State::Digits;
                } else {
                    return Err(Error::unexpected("'-' or '0'..'9'", c, endpos - 1));
                },
                State::Zero => if c == expected_end {
                    success = true;
                    break;
                } else {
                    return Err(Error::unexpected(
                        &format!("{:?}", expected_end),
                        c,
                        endpos - 1,
                    ));
                },
                State::Sign => if c >= '1' && c <= '9' {
                    state = State::Digits;
                } else {
                    return Err(Error::unexpected("'1'..'9'", c, endpos - 1));
                },
                State::Digits => if c >= '0' && c <= '9' {
                    // do nothing, this is ok
                } else if c == expected_end {
                    success = true;
                    break;
                } else {
                    return Err(Error::unexpected(
                        &format!("{:?} or '0'..'9'", expected_end),
                        c,
                        endpos - 1,
                    ));
                },
            }
        }
        if success {
            let slice = &self.source[self.offset..endpos-1];
            assert!(slice.len() >= 0);
            self.offset = endpos;
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

        eprintln!("State: {:?}", self.state.peek());
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
                self.state.push(Seq);
            }
            (Some(&MapValue(label)), Dict) => {
                self.state.replace_top(MapKey(Some(label)));
                self.state.push(MapKey(None));
            }
            (Some(&MapValue(label)), _) => {
                self.state.replace_top(MapKey(Some(label)));
            }
            (_, List) => {
                self.state.push(Seq);
            }
            (_, Dict) => {
                self.state.push(MapKey(None));
            }
            (_, _) => (),
        }
        Ok(Some(tok))
    }

    pub fn tokens(self) -> Tokens<'ser> {
        Tokens(self)
    }
}

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
    pub fn next<'obj>(&'obj mut self) -> Result<Option<Object<'obj, 'ser>>, Error> {
        use self::Token::*;
        Ok(match self.next_token()? {
            None => None,
            Some(End) => None,
            Some(List) => Some(Object::List(ListDecoder::new(self))),
            Some(Dict) => Some(Object::Dict(DictDecoder::new(self))),
            Some(String(s)) => Some(Object::Bytes(s)),
            Some(Num(s)) => Some(Object::Integer(s)),
        })
    }
}

// The option is set to None when the object ends
pub struct DictDecoder<'obj, 'ser: 'obj> {
    decoder: &'obj mut Decoder<'ser>,
    finished: bool,
    start_point: usize,
}
pub struct ListDecoder<'obj, 'ser: 'obj> {
    decoder: &'obj mut Decoder<'ser>,
    finished: bool,
    start_point: usize,
}

impl<'obj, 'ser: 'obj> DictDecoder<'obj, 'ser> {
    fn new(decoder: &'obj mut Decoder<'ser>) -> Self {
        let offset = decoder.offset - 1;
        DictDecoder {
            decoder: decoder,
            finished: false,
            start_point: offset,
        }
    }

    pub fn next<'item>(
        &'item mut self,
    ) -> Result<Option<(&'ser [u8], Object<'item, 'ser>)>, Error> {
        if self.finished {
            return Ok(None);
        }

        // We convert to a token to release the mut ref to decoder
        let key = self.decoder.next()?.map(Object::into_token);

        if let Some(Token::String(k)) = key {
            // This unwrap should be safe because None would produce an error here
            let start = self.decoder.offset;
            let v = self.decoder.next()?.unwrap();
            Ok(Some((k, v)))
        } else {
            // We can't have gotten anything but a string, as anything else would be
            // a state error
            self.finished = true;
            Ok(None)
        }
    }

    pub fn consume_all(&mut self) -> Result<(), Error> {
        while let Some(_) = self.next()? {
            // just drop the items
        }
        Ok(())
    }

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
            decoder: decoder,
            finished: false,
            start_point: offset,
        }
    }

    pub fn next<'item>(&'item mut self) -> Result<Option<Object<'item, 'ser>>, Error> {
        if self.finished {
            return Ok(None);
        }

        let item = self.decoder.next()?;
        if item.is_none() {
            self.finished = true;
        }

        Ok(item)
    }

    pub fn consume_all(&mut self) -> Result<(), Error> {
        while let Some(_) = self.next()? {
            // just drop the items
        }
        Ok(())
    }

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

    static SIMPLE_MSG: &'static [u8] = b"d3:bari1e3:fooli2ei3eee";

    fn decode_tokens(msg: &[u8]) -> Vec<Token> {
        let tokens: Vec<Result<Token, Error>> = Decoder::new(msg).tokens().collect();
        if tokens.iter().all(Result::is_ok) {
            tokens.into_iter().map(Result::unwrap).collect()
        } else {
            panic!("Unexpected tokenization error. Received tokens: {:?}", tokens);

        }
    }

    fn decode_err(msg: &[u8]) -> Error {
        let mut tokens: Vec<Result<Token, Error>> = Decoder::new(msg).tokens().collect();
        if tokens.iter().all(Result::is_ok) {
            panic!("Unexpected parse success: {:?}", tokens);
        } else {
            tokens.pop().unwrap().err().unwrap()
        }
    }

    #[test]
    fn simple_bdecode_tokenization() {
        use super::Token::*;
        let mut tokens: Vec<_> = decode_tokens(SIMPLE_MSG);
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
    fn short_message_should_fail() {
        decode_err(b"d");
    }

    #[test]
    fn map_keys_must_be_strings() {
        decode_err(b"d3:fooi1iei2ei3ee");
    }

    #[test]
    fn map_keys_must_ascend() {
        decode_err(b"d3:fooi1e3:bari1ee");
    }

    #[test]
    fn map_keys_must_be_unique() {
        decode_err(b"d3:fooi1e3:fooi1ee");
    }



    #[test]
    fn dict_drop_should_consume_struct() {
        let mut decoder = Decoder::new(b"d3:fooi1e3:quxi2ee");
        drop(decoder.next());
        assert!(decoder.next().unwrap().is_none())
    }

    #[test]
    fn list_drop_should_consume_struct() {
        let mut decoder = Decoder::new(b"li1ei2ei3ee");
        drop(decoder.next());
        assert!(decoder.next().unwrap().is_none())
    }
}
