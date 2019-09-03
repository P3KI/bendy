//! Port of https://github.com/jamesleonis/bencode-cljc/blob/master/test/bencode_cljc/core_test.cljc
//!
//! Should only use #![no_std] compatible features but still requires the
//! `std` feature flag to avoid that we need to define a global allocator.

extern crate alloc;
use alloc::collections::BTreeMap;

use bendy::{
    decoding::{Error as DecodingError, FromBencode, Object},
    encoding::{Error as EncodingError, SingleItemEncoder, ToBencode},
};

// -----------------------------------------------------------------------------
// Macros
// -----------------------------------------------------------------------------

macro_rules! list(
    {} => { Vec::<Something>::new() };
    { $($value:expr),+ } => {
        {
            let mut list = Vec::new();
            $( list.push(Something::from($value)); )+

            list
        }
     };
);

macro_rules! map(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut map = BTreeMap::new();
            $( map.insert($key.to_owned(), Something::from($value)); )+

            map
        }
     };
);

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[test]
fn string_test_pairs() -> Result<(), Error> {
    let pairs = [
        ("", "0:"),
        ("hello", "5:hello"),
        ("goodbye", "7:goodbye"),
        ("hello world", "11:hello world"),
        ("1-5%3~]+=\\| []>.,`??", "20:1-5%3~]+=\\| []>.,`??"),
    ];

    for (original, expected_encoding) in &pairs {
        let encoded = original.to_bencode()?;
        assert_eq!(expected_encoding.as_bytes(), encoded.as_slice());

        let decoded = String::from_bencode(&encoded)?;
        assert_eq!(original, &decoded);
    }

    Ok(())
}

#[test]
fn integer_test_pairs() -> Result<(), Error> {
    let pairs = [
        (0, "i0e"),
        (5, "i5e"),
        (-5, "i-5e"),
        (005, "i5e"),
        (-005, "i-5e"),
        (1234567890, "i1234567890e"),
        (-1234567890, "i-1234567890e"),
        (i64::max_value(), "i9223372036854775807e"),
        (i64::min_value(), "i-9223372036854775808e"),
    ];
    // Bendy currently doesn't contain a big number implementation..
    //
    // (
    //     123456789012345678901234567890123456789012345678901234567890,
    //     "i123456789012345678901234567890123456789012345678901234567890e"
    // ),
    // (
    //     -123456789012345678901234567890123456789012345678901234567890,
    //     "i-123456789012345678901234567890123456789012345678901234567890e"
    // )

    for (original, expected_encoding) in &pairs {
        let encoded = original.to_bencode()?;
        assert_eq!(expected_encoding.as_bytes(), encoded.as_slice());

        let decoded = i64::from_bencode(&encoded)?;
        assert_eq!(original, &decoded);
    }

    Ok(())
}

#[test]
fn list_test_pairs() -> Result<(), Error> {
    let pairs = [
        (list![], "le"),
        (list!["abra", "cadabra"], "l4:abra7:cadabrae"),
        (list!["spam", "eggs"], "l4:spam4:eggse"),
        (
            list![vec!["list", "of", "lists"], vec!["like", "omygawd!"]],
            "ll4:list2:of5:listsel4:like8:omygawd!ee",
        ),
    ];

    for (original, expected_encoding) in &pairs {
        let encoded = original.to_bencode()?;
        assert_eq!(expected_encoding.as_bytes(), encoded.as_slice());

        let decoded = Vec::<Something>::from_bencode(&encoded)?;
        assert_eq!(original, &decoded);
    }

    Ok(())
}

#[test]
fn map_test_pairs() -> Result<(), Error> {
    let pairs = [
        (BTreeMap::new(), "de"),
        (
            map! {"cow" => "moo", "spam" => "eggs"},
            "d3:cow3:moo4:spam4:eggse",
        ),
        (
            map! {"cow" => "moo", "dog" => "bark"},
            "d3:cow3:moo3:dog4:barke",
        ),
        (
            map! {"dog" => "bark", "cow" => "moo"},
            "d3:cow3:moo3:dog4:barke",
        ),
        (
            map! {"first" => "first", "2ace" => "second", "3ace" => "third"},
            "d4:2ace6:second4:3ace5:third5:first5:firste",
        ),
        (
            map! {"Goodbye" => map! {"maps" => "that don't work", "number" => 100}},
            "d7:Goodbyed4:maps15:that don't work6:numberi100eee",
        ),
        (
            map! {
            "publisher" => "bob", "publisher-webpage" => "www.example.com",
            "publisher.location" => "home"
            },
            "d9:publisher3:bob17:publisher-webpage15:www.example.com18:publisher.location4:homee",
        ),
    ];

    for (original, expected_encoding) in &pairs {
        let encoded = original.to_bencode()?;
        assert_eq!(expected_encoding.as_bytes(), encoded.as_slice());

        let decoded = BTreeMap::<String, Something>::from_bencode(&encoded)?;
        assert_eq!(original, &decoded);
    }

    Ok(())
}

#[test]
fn mixed_use_list_pairs() -> Result<(), Error> {
    let pairs = [(
        list![0, "heterogeneous", -5, "lists", 10, map! {"map" => "well"}],
        "li0e13:heterogeneousi-5e5:listsi10ed3:map4:wellee",
    )];

    for (original, expected_encoding) in &pairs {
        let encoded = original.to_bencode()?;
        assert_eq!(expected_encoding.as_bytes(), encoded.as_slice());

        let decoded = Vec::<Something>::from_bencode(&encoded)?;
        assert_eq!(original, &decoded);
    }

    Ok(())
}

#[test]
fn mixed_use_dict_pairs() -> Result<(), Error> {
    let pairs = [
        (
            map! {
		"hello" => list!["world!", "gaia!", "mother earth!"],
		"Goodbye" => map! {"maps" => "that don't work", "number" => 100}
	    },
            "d7:Goodbyed4:maps15:that don't work6:numberi100ee5:hellol6:world!5:gaia!13:mother earth!ee"
        ),
        (
             map! {"hello" => list!["world!", "gaia!", "mother earth!"]},
             "d5:hellol6:world!5:gaia!13:mother earth!ee"
        ),
        (
            map! {"spam" => list!["a", "b"]}, "d4:spaml1:a1:bee"),
        (
            map! {
                "t" => "aa", "y" => "q", "q" => "ping",
                "a" => map! { "id" => "abcdefghij0123456789" }
            },
            "d1:ad2:id20:abcdefghij0123456789e1:q4:ping1:t2:aa1:y1:qe",
        ),
        (
            map! {
                "t" => "aa", "y" => "q", "q" => "find_node",
                "a" => map! { "id" => "abcdefghij0123456789", "target" => "mnopqrstuvwxyz123456" }
            },
            "d1:ad2:id20:abcdefghij01234567896:target20:mnopqrstuvwxyz123456e1:q9:find_node1:t2:aa1:y1:qe"
        ),
        (
            map! {
                "t" => "aa", "y" => "q", "q" => "get_peers",
                "a" => map! { "id" => "abcdefghij0123456789", "info_hash" => "mnopqrstuvwxyz123456" }
            },
            "d1:ad2:id20:abcdefghij01234567899:info_hash20:mnopqrstuvwxyz123456e1:q9:get_peers1:t2:aa1:y1:qe"
        ),
        (
            map! {
                "t" => "aa", "y" => "r",
                "r" => map! {
			"id" => "abcdefghij0123456789",
			"token" => "aoeusnth", "values" => vec!["axje.u", "idhtnm"]
		}
            },
            "d1:rd2:id20:abcdefghij01234567895:token8:aoeusnth6:valuesl6:axje.u6:idhtnmee1:t2:aa1:y1:re"
        )
    ];

    for (original, expected_encoding) in &pairs {
        let encoded = original.to_bencode()?;
        assert_eq!(expected_encoding.as_bytes(), encoded.as_slice());

        let decoded = BTreeMap::<String, Something>::from_bencode(&encoded)?;
        assert_eq!(original, &decoded);
    }

    Ok(())
}

#[test]
fn illegal_integer_encodings() {
    let values = [
        "i-0e", "i09e", "i-09e", "i-0123e", "i-00123e", "i0123e", "i00123e", "i12-345", "i-12-345",
        "i-1", "i1",
    ];
    // Bendy currently doesn't fail if it encounters unused tokens
    //
    // "i12345ei10e5:eoeoee",
    // "i-12345ei10e5:eoeoee"

    for value in &values {
        let error = i64::from_bencode(value.as_bytes()).unwrap_err();
        assert!(error.to_string().contains("encoding corrupted"));
    }
}

#[test]
fn illegal_string_encodings() {
    let values = [":hello", "-5:hello", "-5:", "5:", "10:hello"];
    // Bendy currently doesn't fail if it encounters unused tokens
    //
    // "5:hello5:hello",
    // "5:helloi10e",
    // 10:hello5:hello",
    // "10:helloi0e",
    // "10:helloi123456789e"

    for value in &values {
        let error = String::from_bencode(value.as_bytes()).unwrap_err();
        assert!(error.to_string().contains("encoding corrupted"));
    }
}

#[test]
fn illegal_list_encodings() {
    let values = [
        "l",
        "lsde",
        "li10e5hello",
        "l10:helloi123456789ee",
        "l10:helloi123456789e5:helloe",
        "l5:helloi123456789e10:helloe",
    ];
    // Bendy currently doesn't fail if it encounters unused tokens
    //
    // "l5:hello5:worldei10e",

    for value in &values {
        let error = Vec::<Something>::from_bencode(value.as_bytes()).unwrap_err();
        assert!(error.to_string().contains("encoding corrupted"));
    }
}

#[test]
fn illegal_dictionary_encodings() {
    let values = [
        "d",
        "duuuuure",
        "d5:hello5:world",
        "d10:helloi123456789ee",
        "d5:helloi123456789e5:helloe",
        "di10e5:hello5:worldi10ee",
        "d5:worldi10ei10e5:helloe",
        "dle5:hello5:worldi10ee",
        "dli10ei11ee5:hello5:worldi10ee",
        "dde5:hello5:worldi10ee",
        "dd8:innermapi11ee5:hello5:worldi10ee",
    ];
    // Bendy currently doesn't fail if it encounters unused tokens
    //
    // "d5:hello5:worldei10e",

    for value in &values {
        let error = BTreeMap::<String, Something>::from_bencode(value.as_bytes()).unwrap_err();
        assert!(error.to_string().contains("encoding corrupted"));
    }
}

// -----------------------------------------------------------------------------
// Dynamic Typing Utility
// -----------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
enum Something {
    Bytes(String),
    Dict(BTreeMap<String, Something>),
    Integer(i64),
    List(Vec<Something>),
}

impl From<&str> for Something {
    fn from(content: &str) -> Self {
        Something::Bytes(content.to_owned())
    }
}

impl<ContentT> From<BTreeMap<String, ContentT>> for Something
where
    Something: From<ContentT>,
{
    fn from(content: BTreeMap<String, ContentT>) -> Self {
        let content = content
            .into_iter()
            .map(|(key, value)| (key, value.into()))
            .collect();

        Something::Dict(content)
    }
}

impl From<i64> for Something {
    fn from(content: i64) -> Self {
        Something::Integer(content)
    }
}

impl<ContentT> From<Vec<ContentT>> for Something
where
    Something: From<ContentT>,
{
    fn from(content: Vec<ContentT>) -> Self {
        let content = content.into_iter().map(Into::into).collect();
        Something::List(content)
    }
}

impl FromBencode for Something {
    fn decode_bencode_object(object: Object) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let something = match object {
            Object::Bytes(content) => {
                Something::Bytes(String::from_utf8_lossy(content).to_string())
            },
            Object::Integer(number) => Something::Integer(number.parse().unwrap()),
            object @ Object::Dict(_) => {
                Something::Dict(BTreeMap::decode_bencode_object(object).unwrap())
            },
            object @ Object::List(_) => {
                Something::List(Vec::decode_bencode_object(object).unwrap())
            },
        };

        Ok(something)
    }
}

impl ToBencode for Something {
    const MAX_DEPTH: usize = 999;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), EncodingError> {
        match self {
            Something::Bytes(content) => encoder.emit(content),
            Something::Dict(content) => encoder.emit(content),
            Something::Integer(content) => encoder.emit(content),
            Something::List(content) => encoder.emit(content),
        }
    }
}

// -----------------------------------------------------------------------------
// Error
// -----------------------------------------------------------------------------

#[derive(Debug)]
enum Error {
    DecodingError(DecodingError),
    EncodingError(EncodingError),
}

impl From<DecodingError> for Error {
    fn from(error: DecodingError) -> Self {
        Error::DecodingError(error)
    }
}

impl From<EncodingError> for Error {
    fn from(error: EncodingError) -> Self {
        Error::EncodingError(error)
    }
}
