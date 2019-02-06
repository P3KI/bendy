use failure::Fail;

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

impl<'a> Token<'a> {
    pub fn name(&self) -> &'static str {
        match *self {
            Token::Dict => "Dict",
            Token::End => "End",
            Token::List => "List",
            Token::Num(_) => "Num",
            Token::String(_) => "String",
        }
    }
}

/// An encoding or decoding error
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Fail)]
pub enum Error {
    #[fail(display = "Saw the wrong type of token: {}", _0)]
    /// Wrong type of token detected.
    InvalidState(String),
    #[fail(display = "Keys were not sorted")]
    /// Keys were not sorted.
    UnsortedKeys,
    #[fail(display = "Reached EOF in the middle of a message")]
    /// EOF reached to early.
    UnexpectedEof,
    #[fail(display = "Malformed number of unexpected character: {}", _0)]
    /// Unexpected characters detected.
    SyntaxError(String),
    #[fail(display = "Maximum nesting depth exceeded")]
    /// Exceeded the recursion limit.
    NestingTooDeep,
}

impl Error {
    pub fn unexpected(expected: &str, got: char, offset: usize) -> Self {
        Error::SyntaxError(format!(
            "Expected {}, got {:?} at offset {}",
            expected, got, offset
        ))
    }

    pub fn invalid_state(expected: &str) -> Self {
        Error::InvalidState(expected.to_owned())
    }
}
