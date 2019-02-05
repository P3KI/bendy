use crate::{
    decoding::{DictDecoder, ListDecoder},
    token::Token,
};

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
    pub fn into_token(self) -> Token<'ser> {
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
