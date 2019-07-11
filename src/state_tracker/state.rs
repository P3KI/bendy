#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::state_tracker::{Stack, StructureError, Token};

/// The state of current level of the decoder
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
enum State<S: AsRef<[u8]>, E> {
    /// An inner list. Allows any token
    Seq,
    /// Inside a map, expecting a key. Contains the last key read, so sorting can be validated
    MapKey(Option<S>),
    /// Inside a map, expecting a value. Contains the last key read, so sorting can be validated
    MapValue(S),
    /// Received an error while decoding
    Failed(E),
}

/// Used to validate that a structure is valid
#[derive(Debug)]
pub struct StateTracker<S: AsRef<[u8]>, E = StructureError> {
    state: Vec<State<S, E>>,
    max_depth: usize,
}

impl<S: AsRef<[u8]>, E> Default for StateTracker<S, E> {
    fn default() -> Self {
        StateTracker {
            state: Vec::new(),
            max_depth: 2048,
        }
    }
}

impl<S: AsRef<[u8]>, E> StateTracker<S, E>
where
    S: AsRef<[u8]>,
    E: From<StructureError> + Clone,
{
    pub fn new() -> Self {
        <Self as Default>::default()
    }

    pub fn set_max_depth(&mut self, new_max_depth: usize) {
        self.max_depth = new_max_depth
    }

    pub fn remaining_depth(&self) -> usize {
        self.max_depth - self.state.len()
    }

    /// Observe that an EOF was seen. This function is idempotent.
    pub fn observe_eof(&mut self) -> Result<(), E> {
        self.check_error()?;

        if self.state.is_empty() {
            Ok(())
        } else {
            self.latch_err(Err(E::from(StructureError::UnexpectedEof)))
        }
    }

    #[allow(clippy::match_same_arms)]
    pub fn observe_token<'a>(&mut self, token: &Token<'a>) -> Result<(), E>
    where
        S: From<&'a [u8]>,
    {
        use self::{State::*, Token::*};

        match (self.state.pop(), *token) {
            (None, End) => {
                return self.latch_err(Err(E::from(StructureError::invalid_state(
                    "End not allowed at top level",
                ))));
            },
            (Some(Seq), End) => {},
            (Some(MapKey(_)), End) => {},
            (Some(MapKey(None)), String(label)) => {
                self.state.push(MapValue(S::from(label)));
            },
            (Some(MapKey(Some(oldlabel))), String(label)) => {
                if oldlabel.as_ref() >= label {
                    return self.latch_err(Err(E::from(StructureError::UnsortedKeys)));
                }
                self.state.push(MapValue(S::from(label)));
            },
            (Some(oldstate @ MapKey(_)), _tok) => {
                self.state.push(oldstate);
                return self.latch_err(Err(E::from(StructureError::invalid_state(
                    "Map keys must be strings",
                ))));
            },
            (Some(MapValue(label)), List) => {
                self.state.push(MapKey(Some(label)));
                if self.state.len() >= self.max_depth {
                    return self.latch_err(Err(E::from(StructureError::NestingTooDeep)));
                }
                self.state.push(Seq);
            },
            (Some(MapValue(label)), Dict) => {
                self.state.push(MapKey(Some(label)));
                if self.state.len() >= self.max_depth {
                    return self.latch_err(Err(E::from(StructureError::NestingTooDeep)));
                }
                self.state.push(MapKey(None));
            },
            (Some(oldstate @ MapValue(_)), End) => {
                self.state.push(oldstate);
                return self.latch_err(Err(E::from(StructureError::invalid_state(
                    "Missing map value",
                ))));
            },
            (Some(MapValue(label)), _) => {
                self.state.push(MapKey(Some(label)));
            },
            (oldstate, List) => {
                if let Some(oldstate) = oldstate {
                    self.state.push(oldstate);
                }
                if self.state.len() >= self.max_depth {
                    return self.latch_err(Err(E::from(StructureError::NestingTooDeep)));
                }
                self.state.push(Seq);
            },
            (oldstate, Dict) => {
                if let Some(oldstate) = oldstate {
                    self.state.push(oldstate);
                }

                if self.state.len() >= self.max_depth {
                    return self.latch_err(Err(E::from(StructureError::NestingTooDeep)));
                }
                self.state.push(MapKey(None));
            },
            (oldstate, _) => {
                if let Some(oldstate) = oldstate {
                    self.state.push(oldstate);
                }
            },
        }
        Ok(())
    }

    pub fn latch_err<T>(&mut self, result: Result<T, E>) -> Result<T, E> {
        self.check_error()?;
        if let Err(ref err) = result {
            self.state.push(State::Failed(err.clone()))
        }
        result
    }

    pub fn check_error(&self) -> Result<(), E> {
        if let Some(&State::Failed(ref error)) = self.state.peek() {
            Err(error.clone())
        } else {
            Ok(())
        }
    }
}
