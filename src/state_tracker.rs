use super::Error;

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

/// The state of current level of the decoder
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum DecodeState<S: AsRef<[u8]>> {
    /// An inner list. Allows any token
    Seq,
    /// Inside a map, expecting a key. Contains the last key read, so sorting can be validated
    MapKey(Option<S>),
    /// Inside a map, expecting a value. Contains the last key read, so sorting can be validated
    MapValue(S),
    /// Received an error while decoding
    Failed(Error),
}

/// Used to validate that a structure is valid
pub struct StateTracker<S: AsRef<[u8]>> {
    state: Vec<DecodeState<S>>,
    max_depth: usize,
}

impl<S: AsRef<[u8]>> StateTracker<S> {
    pub fn new() -> Self {
        StateTracker{
            state: Vec::new(),
            max_depth: 2048,
        }
    }

    pub fn set_max_depth(&mut self, new_max_depth: usize) {
        self.max_depth = new_max_depth
    }

    pub fn observe_eof(&mut self) -> Result<(), Error> {
        if self.state.is_empty() {
            return Ok(());
        } else {
            return self.latch_err(Err(Error::UnexpectedEof));
        }
    }

    pub fn observe_token<'a>(&mut self, token: &Token<'a>) -> Result<(), Error>
    where S: From<&'a [u8]> {
        use self::Token::*;
        use self::DecodeState::*;

        match (self.state.pop(), *token) {
            (None, End) => {
                return self.latch_err(Err(Error::InvalidState(
                    "End not allowed at top level".to_owned(),
                )));
            }
            (Some(Seq), End) => {
            }
            (Some(MapKey(_)), End) => {
            }
            (Some(MapKey(None)), String(label)) => {
                self.state.push(MapValue(S::from(label)));
            }
            (Some(MapKey(Some(oldlabel))), String(label)) => {
                if oldlabel.as_ref() >= label {
                    return self.latch_err(Err(Error::UnsortedKeys));
                }
                self.state.push(MapValue(S::from(label)));
            }
            (Some(oldstate@MapKey(_)), _tok) => {
                self.state.push(oldstate);
                return self.latch_err(Err(Error::InvalidState(
                    "Map keys must be strings".to_owned(),
                )));
            }
            (Some(MapValue(label)), List) => {
                self.state.push(MapKey(Some(label)));
                if self.state.len() >= self.max_depth {
                    return self.latch_err(Err(Error::NestingTooDeep))
                }
                self.state.push(Seq);
            }
            (Some(MapValue(label)), Dict) => {
                self.state.push(MapKey(Some(label)));
                if self.state.len() >= self.max_depth {
                    return self.latch_err(Err(Error::NestingTooDeep))
                }
                self.state.push(MapKey(None));
            }
            (Some(oldstate@MapValue(_)), End) => {
                self.state.push(oldstate);
                return self.latch_err(Err(Error::InvalidState("Missing map value".to_owned())))
            }
            (Some(MapValue(label)), _) => {
                self.state.push(MapKey(Some(label)));
            }
            (oldstate, List) => {
                if let Some(oldstate) = oldstate {
                    self.state.push(oldstate);
                }
                if self.state.len() >= self.max_depth {
                    return self.latch_err(Err(Error::NestingTooDeep))
                }
                self.state.push(Seq);
            }
            (oldstate, Dict) => {
                if let Some(oldstate) = oldstate {
                    self.state.push(oldstate);
                }

                if self.state.len() >= self.max_depth {
                    return self.latch_err(Err(Error::NestingTooDeep))
                }
                self.state.push(MapKey(None));
            }
            (oldstate, _) => {
                if let Some(oldstate) = oldstate {
                    self.state.push(oldstate);
                }
            }
            ,
        }
        Ok(())
    }

    pub fn latch_err<T>(&mut self, result: Result<T, Error>) -> Result<T, Error> {
        if let Err(ref err) = result {
            self.state.push(DecodeState::Failed(err.clone()))
        }
        result
    }

    pub fn check_error(&self) -> Result<(), Error> {
        if let Some(&DecodeState::Failed(ref error)) = self.state.peek() {
            Err(error.clone())
        } else {
            Ok(())
        }
    }
}