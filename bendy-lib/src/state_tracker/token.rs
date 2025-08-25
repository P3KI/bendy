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
