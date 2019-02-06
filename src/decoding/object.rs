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
    /// [`Ok(v)`]. Any other variant returns the given default value.
    ///
    /// Default arguments passed into `bytes_or` are eagerly evaluated; if you
    /// are passing the result of a function call, it is recommended to use
    /// [`bytes_or_else`], which is lazily evaluated.
    ///
    /// [`Object::Bytes(v)`]: self::Object::Bytes
    /// [`Ok(v)`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Ok
    /// [`bytes_or_else`]: self::Object::bytes_or_else
    ///
    /// # Examples
    ///
    /// ```
    /// use bendy::decoder::Object;
    ///
    /// let x = Object::Bytes(b"foo");
    /// assert_eq!(Ok(&b"foo"[..]), x.bytes_or(Err("failure")));
    ///
    /// let x = Object::Integer("foo");
    /// assert_eq!(Err("failure"), x.bytes_or(Err("failure")));
    /// ```
    pub fn bytes_or<ErrorT>(
        self,
        default: Result<&'ser [u8], ErrorT>,
    ) -> Result<&'ser [u8], ErrorT> {
        match self {
            Object::Bytes(content) => Ok(content),
            _ => default,
        }
    }

    /// Try to treat the object as a byte string, mapping [`Object::Bytes(v)`] into
    /// [`Ok(v)`]. Any other variant is passed into the given fallback method.
    ///
    /// [`Object::Bytes(v)`]: self::Object::Bytes
    /// [`Ok(v)`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Ok
    ///
    /// # Examples
    ///
    /// ```
    /// use bendy::decoder::Object;
    ///
    /// let x = Object::Bytes(b"foo");
    /// assert_eq!(
    ///     Ok(&b"foo"[..]),
    ///     x.bytes_or_else(|obj| Err(obj.into_token().name()))
    /// );
    ///
    /// let x = Object::Integer("foo");
    /// assert_eq!(
    ///     Err("Num"),
    ///     x.bytes_or_else(|obj| Err(obj.into_token().name()))
    /// );
    /// ```
    pub fn bytes_or_else<ErrorT>(
        self,
        op: impl FnOnce(Self) -> Result<&'ser [u8], ErrorT>,
    ) -> Result<&'ser [u8], ErrorT> {
        match self {
            Object::Bytes(content) => Ok(content),
            _ => op(self),
        }
    }

    /// Try to treat the object as an integer and return the internal string representation,
    /// mapping [`Object::Integer(v)`] into [`Ok(v)`]. Any other variant returns the given
    /// default value.
    ///
    /// Default arguments passed into `integer_or` are eagerly evaluated; if you are passing
    /// the result of a function call, it is recommended to use [`integer_or_else`], which
    /// is lazily evaluated.
    ///
    /// [`Object::Integer(v)`]: self::Object::Integer
    /// [`Ok(v)`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Ok
    /// [`integer_or_else`]: self::Object::integer_or_else
    ///
    /// # Examples
    ///
    /// ```
    /// use bendy::decoder::Object;
    ///
    /// let x = Object::Integer("123");
    /// assert_eq!(Ok(&"123"[..]), x.integer_or(Err("failure")));
    ///
    /// let x = Object::Bytes(b"foo");
    /// assert_eq!(Err("failure"), x.integer_or(Err("failure")));
    /// ```
    pub fn integer_or<ErrorT>(
        self,
        default: Result<&'ser str, ErrorT>,
    ) -> Result<&'ser str, ErrorT> {
        match self {
            Object::Integer(content) => Ok(content),
            _ => default,
        }
    }

    /// Try to treat the object as an integer and return the internal string representation,
    /// mapping [`Object::Integer(v)`] into [`Ok(v)`]. Any other variant is passed into the
    /// given fallback method.
    ///
    /// [`Object::Integer(v)`]: self::Object::Integer
    /// [`Ok(v)`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Ok
    ///
    /// # Examples
    ///
    /// ```
    /// use bendy::decoder::Object;
    ///
    /// let x = Object::Integer("123");
    /// assert_eq!(
    ///     Ok(&"123"[..]),
    ///     x.integer_or_else(|obj| Err(obj.into_token().name()))
    /// );
    ///
    /// let x = Object::Bytes(b"foo");
    /// assert_eq!(
    ///     Err("String"),
    ///     x.integer_or_else(|obj| Err(obj.into_token().name()))
    /// );
    /// ```
    pub fn integer_or_else<ErrorT>(
        self,
        op: impl FnOnce(Self) -> Result<&'ser str, ErrorT>,
    ) -> Result<&'ser str, ErrorT> {
        match self {
            Object::Integer(content) => Ok(content),
            _ => op(self),
        }
    }

    /// Try to treat the object as a list and return the internal list content decoder,
    /// mapping [`Object::List(v)`] into [`Ok(v)`]. Any other variant returns the given
    /// default value.
    ///
    /// Default arguments passed into `list_or` are eagerly evaluated; if you are passing
    /// the result of a function call, it is recommended to use [`list_or_else`], which is
    /// lazily evaluated.
    ///
    /// [`Object::List(v)`]: self::Object::List
    /// [`Ok(v)`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Ok
    /// [`list_or_else`]: self::Object::list_or_else
    ///
    /// # Examples
    ///
    /// ```
    /// use bendy::decoder::{Decoder, Object};
    ///
    /// let mut list_decoder = Decoder::new(b"le");
    /// let x = list_decoder.next_object().unwrap().unwrap();
    ///
    /// assert!(x.list_or(Err("failure")).is_ok());
    ///
    /// let x = Object::Bytes(b"foo");
    /// assert_eq!("failure", x.list_or(Err("failure")).unwrap_err());
    /// ```
    pub fn list_or<ErrorT>(
        self,
        default: Result<ListDecoder<'obj, 'ser>, ErrorT>,
    ) -> Result<ListDecoder<'obj, 'ser>, ErrorT> {
        match self {
            Object::List(content) => Ok(content),
            _ => default,
        }
    }

    /// Try to treat the object as a list and return the internal list content decoder,
    /// mapping [`Object::List(v)`] into [`Ok(v)`]. Any other variant is passed into the
    /// given fallback method.
    ///
    /// [`Object::List(v)`]: self::Object::List
    /// [`Ok(v)`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Ok
    ///
    /// # Examples
    ///
    /// ```
    /// use bendy::decoder::{Decoder, Object};
    ///
    /// let mut list_decoder = Decoder::new(b"le");
    /// let x = list_decoder.next_object().unwrap().unwrap();
    ///
    /// assert!(x.list_or_else(|obj| Err(obj.into_token().name())).is_ok());
    ///
    /// let x = Object::Bytes(b"foo");
    /// assert_eq!(
    ///     "String",
    ///     x.list_or_else(|obj| Err(obj.into_token().name()))
    ///         .unwrap_err()
    /// );
    /// ```
    pub fn list_or_else<ErrorT>(
        self,
        op: impl FnOnce(Self) -> Result<ListDecoder<'obj, 'ser>, ErrorT>,
    ) -> Result<ListDecoder<'obj, 'ser>, ErrorT> {
        match self {
            Object::List(content) => Ok(content),
            _ => op(self),
        }
    }

    /// Try to treat the object as a dictionary and return the internal dictionary content
    /// decoder, mapping [`Object::Dict(v)`] into [`Ok(v)`]. Any other variant returns the
    /// given default value.
    ///
    /// Default arguments passed to `dictionary_or` are eagerly evaluated; if you are passing
    /// the result of a function call, it is recommended to use [`dictionary_or_else`], which
    /// is lazily evaluated.
    ///
    /// [`Object::Dict(v)`]: self::Object::Dict
    /// [`Ok(v)`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Ok
    /// [`dictionary_or_else`]: self::Object::dictionary_or_else
    ///
    /// # Examples
    ///
    /// ```
    /// use bendy::decoder::{Decoder, Object};
    ///
    /// let mut dict_decoder = Decoder::new(b"de");
    /// let x = dict_decoder.next_object().unwrap().unwrap();
    ///
    /// assert!(x.dictionary_or(Err("failure")).is_ok());
    ///
    /// let x = Object::Bytes(b"foo");
    /// assert_eq!("failure", x.dictionary_or(Err("failure")).unwrap_err());
    /// ```
    pub fn dictionary_or<ErrorT>(
        self,
        default: Result<DictDecoder<'obj, 'ser>, ErrorT>,
    ) -> Result<DictDecoder<'obj, 'ser>, ErrorT> {
        match self {
            Object::Dict(content) => Ok(content),
            _ => default,
        }
    }

    /// Try to treat the object as a dictionary and return the internal dictionary content
    /// decoder, mapping [`Object::Dict(v)`] into [`Ok(v)`]. Any other variant is passed
    /// into the given fallback method.
    ///
    /// [`Object::Dict(v)`]: self::Object::Dict
    /// [`Ok(v)`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Ok
    ///
    /// # Examples
    ///
    /// ```
    /// use bendy::decoder::{Decoder, Object};
    ///
    /// let mut dict_decoder = Decoder::new(b"de");
    /// let x = dict_decoder.next_object().unwrap().unwrap();
    ///
    /// assert!(x
    ///     .dictionary_or_else(|obj| Err(obj.into_token().name()))
    ///     .is_ok());
    ///
    /// let x = Object::Bytes(b"foo");
    /// assert_eq!(
    ///     "String",
    ///     x.dictionary_or_else(|obj| Err(obj.into_token().name()))
    ///         .unwrap_err()
    /// );
    /// ```
    pub fn dictionary_or_else<ErrorT>(
        self,
        op: impl FnOnce(Self) -> Result<DictDecoder<'obj, 'ser>, ErrorT>,
    ) -> Result<DictDecoder<'obj, 'ser>, ErrorT> {
        match self {
            Object::Dict(content) => Ok(content),
            _ => op(self),
        }
    }
}
