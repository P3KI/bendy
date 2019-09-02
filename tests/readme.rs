// Please keep the code below in sync with `README.md`.
//
// If `cfg(doctest)` gets stablized or `cfg(test)` gets fixed, we can use
// doc-comment for running tests in `README.md`.

mod encoding_1 {
    use bendy::encoding::{Error, ToBencode};

    #[test]
    fn encode_vector() -> Result<(), Error> {
        let my_data = vec!["hello", "world"];
        let encoded = my_data.to_bencode()?;

        assert_eq!(b"l5:hello5:worlde", encoded.as_slice());
        Ok(())
    }
}

mod encoding_2 {
    use bendy::encoding::{Error, SingleItemEncoder, ToBencode};

    struct IntegerWrapper(i64);

    impl ToBencode for IntegerWrapper {
        const MAX_DEPTH: usize = 0;

        fn encode(&self, encoder: SingleItemEncoder) -> Result<(), Error> {
            encoder.emit_int(self.0)
        }
    }

    #[test]
    fn encode_integer() -> Result<(), Error> {
        let example = IntegerWrapper(21);

        let encoded = example.to_bencode()?;
        assert_eq!(b"i21e", encoded.as_slice());

        let encoded = 21.to_bencode()?;
        assert_eq!(b"i21e", encoded.as_slice());

        Ok(())
    }
}

mod encoding_3 {
    use bendy::encoding::{Error, SingleItemEncoder, ToBencode};

    struct StringWrapper(String);

    impl ToBencode for StringWrapper {
        const MAX_DEPTH: usize = 0;

        fn encode(&self, encoder: SingleItemEncoder) -> Result<(), Error> {
            encoder.emit_str(&self.0)
        }
    }

    #[test]
    fn encode_string() -> Result<(), Error> {
        let example = StringWrapper("content".to_string());

        let encoded = example.to_bencode()?;
        assert_eq!(b"7:content", encoded.as_slice());

        let encoded = "content".to_bencode()?;
        assert_eq!(b"7:content", encoded.as_slice());

        Ok(())
    }
}

mod encoding_4 {
    use bendy::encoding::{AsString, Error, SingleItemEncoder, ToBencode};

    struct ByteStringWrapper(Vec<u8>);

    impl ToBencode for ByteStringWrapper {
        const MAX_DEPTH: usize = 0;

        fn encode(&self, encoder: SingleItemEncoder) -> Result<(), Error> {
            let content = AsString(&self.0);
            encoder.emit(&content)
        }
    }

    #[test]
    fn encode_byte_string() -> Result<(), Error> {
        let example = ByteStringWrapper(b"content".to_vec());

        let encoded = example.to_bencode()?;
        assert_eq!(b"7:content", encoded.as_slice());

        let encoded = AsString(b"content").to_bencode()?;
        assert_eq!(b"7:content", encoded.as_slice());

        Ok(())
    }
}

mod encoding_5 {
    use bendy::encoding::{Error, SingleItemEncoder, ToBencode};

    struct Example {
        label: String,
        counter: u64,
    }

    impl ToBencode for Example {
        const MAX_DEPTH: usize = 1;

        fn encode(&self, encoder: SingleItemEncoder) -> Result<(), Error> {
            encoder.emit_dict(|mut e| {
                e.emit_pair(b"counter", &self.counter)?;
                e.emit_pair(b"label", &self.label)?;

                Ok(())
            })
        }
    }

    #[test]
    fn encode_dictionary() -> Result<(), Error> {
        let example = Example {
            label: "Example".to_string(),
            counter: 0,
        };

        let encoded = example.to_bencode()?;
        assert_eq!(b"d7:counteri0e5:label7:Examplee", encoded.as_slice());

        Ok(())
    }
}

mod encoding_6 {
    use bendy::encoding::{Error, SingleItemEncoder, ToBencode};

    struct Location(i64, i64);

    impl ToBencode for Location {
        const MAX_DEPTH: usize = 1;

        fn encode(&self, encoder: SingleItemEncoder) -> Result<(), Error> {
            encoder.emit_list(|e| {
                e.emit_int(self.0)?;
                e.emit_int(self.1)
            })
        }
    }

    #[test]
    fn encode_list() -> Result<(), Error> {
        let example = Location(2, 3);

        let encoded = example.to_bencode()?;
        assert_eq!(b"li2ei3ee", encoded.as_slice());

        Ok(())
    }
}

mod decoding_1 {
    use bendy::decoding::{Error, FromBencode};

    #[test]
    fn decode_vector() -> Result<(), Error> {
        let encoded = b"l5:hello5:worlde".to_vec();
        let decoded = Vec::<String>::from_bencode(&encoded)?;

        assert_eq!(vec!["hello", "world"], decoded);
        Ok(())
    }
}

mod decoding_2 {
    use bendy::decoding::{Error, FromBencode, Object};

    #[derive(Debug, Eq, PartialEq)]
    struct IntegerWrapper(i64);

    impl FromBencode for IntegerWrapper {
        const EXPECTED_RECURSION_DEPTH: usize = 0;

        fn decode_bencode_object(object: Object) -> Result<Self, Error> {
            // This is an example for content handling. It would also be possible
            // to call  `i64::decode_bencode_object(object)` directly.
            let content = object.try_into_integer()?;
            let number = content.parse::<i64>()?;

            Ok(IntegerWrapper(number))
        }
    }

    #[test]
    fn decode_integer() -> Result<(), Error> {
        let encoded = b"i21e".to_vec();

        let example = IntegerWrapper::from_bencode(&encoded)?;
        assert_eq!(IntegerWrapper(21), example);

        let example = i64::from_bencode(&encoded)?;
        assert_eq!(21, example);

        Ok(())
    }
}

mod decoding_3 {
    use bendy::decoding::{Error, FromBencode, Object};

    #[derive(Debug, Eq, PartialEq)]
    struct StringWrapper(String);

    impl FromBencode for StringWrapper {
        const EXPECTED_RECURSION_DEPTH: usize = 0;

        fn decode_bencode_object(object: Object) -> Result<Self, Error> {
            // This is an example for content handling. It would also be possible
            // to call  `String::decode_bencode_object(object)` directly.
            let content = object.try_into_bytes()?;
            let content = String::from_utf8(content.to_vec())?;

            Ok(StringWrapper(content))
        }
    }

    #[test]
    fn decode_string() -> Result<(), Error> {
        let encoded = b"7:content".to_vec();

        let example = StringWrapper::from_bencode(&encoded)?;
        assert_eq!(StringWrapper("content".to_string()), example);

        let example = String::from_bencode(&encoded)?;
        assert_eq!("content".to_string(), example);

        Ok(())
    }
}

mod decoding_4 {
    use bendy::{
        decoding::{Error, FromBencode, Object},
        encoding::AsString,
    };

    #[derive(Debug, Eq, PartialEq)]
    struct ByteStringWrapper(Vec<u8>);

    impl FromBencode for ByteStringWrapper {
        const EXPECTED_RECURSION_DEPTH: usize = 0;

        fn decode_bencode_object(object: Object) -> Result<Self, Error> {
            let content = AsString::decode_bencode_object(object)?;
            Ok(ByteStringWrapper(content.0))
        }
    }

    #[test]
    fn decode_byte_string() -> Result<(), Error> {
        let encoded = b"7:content".to_vec();

        let example = ByteStringWrapper::from_bencode(&encoded)?;
        assert_eq!(ByteStringWrapper(b"content".to_vec()), example);

        let example = AsString::from_bencode(&encoded)?;
        assert_eq!(b"content".to_vec(), example.0);

        Ok(())
    }
}

mod decoding_5 {
    use bendy::decoding::{Error, FromBencode, Object, ResultExt};

    #[derive(Debug, Eq, PartialEq)]
    struct Example {
        label: String,
        counter: u64,
    }

    impl FromBencode for Example {
        const EXPECTED_RECURSION_DEPTH: usize = 1;

        fn decode_bencode_object(object: Object) -> Result<Self, Error> {
            let mut counter = None;
            let mut label = None;

            let mut dict = object.try_into_dictionary()?;
            while let Some(pair) = dict.next_pair()? {
                match pair {
                    (b"counter", value) => {
                        counter = u64::decode_bencode_object(value)
                            .context("counter")
                            .map(Some)?;
                    },
                    (b"label", value) => {
                        label = String::decode_bencode_object(value)
                            .context("label")
                            .map(Some)?;
                    },
                    (unknown_field, _) => {
                        return Err(Error::unexpected_field(String::from_utf8_lossy(
                            unknown_field,
                        )));
                    },
                }
            }

            let counter = counter.ok_or_else(|| Error::missing_field("counter"))?;
            let label = label.ok_or_else(|| Error::missing_field("label"))?;

            Ok(Example { counter, label })
        }
    }

    #[test]
    fn decode_dictionary() -> Result<(), Error> {
        let encoded = b"d7:counteri0e5:label7:Examplee".to_vec();
        let expected = Example {
            label: "Example".to_string(),
            counter: 0,
        };

        let example = Example::from_bencode(&encoded)?;
        assert_eq!(expected, example);

        Ok(())
    }
}

mod decoding_6 {
    use bendy::decoding::{Error, FromBencode, Object};

    #[derive(Debug, PartialEq, Eq)]
    struct Location(i64, i64);

    impl FromBencode for Location {
        const EXPECTED_RECURSION_DEPTH: usize = 1;

        fn decode_bencode_object(object: Object) -> Result<Self, Error> {
            let mut list = object.try_into_list()?;

            let x = list.next_object()?.ok_or(Error::missing_field("x"))?;
            let x = i64::decode_bencode_object(x)?;

            let y = list.next_object()?.ok_or(Error::missing_field("y"))?;
            let y = i64::decode_bencode_object(y)?;

            Ok(Location(x, y))
        }
    }

    #[test]
    fn decode_list() -> Result<(), Error> {
        let encoded = b"li2ei3ee".to_vec();
        let expected = Location(2, 3);

        let example = Location::from_bencode(&encoded)?;
        assert_eq!(expected, example);

        Ok(())
    }
}
