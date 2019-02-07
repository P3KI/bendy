#[macro_use]
extern crate failure;

use bendy::decoding::{Decoder, Object};
use failure::Error;

static SIMPLE_MSG: &'static [u8] = b"d3:bari1e3:fooli2ei3eee";

// test of high-level interface
// Normally, a trait like this would have a better error type
trait DecodeFrom<'ser>: Sized {
    fn decode<'obj>(object: Object<'obj, 'ser>) -> Result<Self, Error>;
}

impl<'ser, T: DecodeFrom<'ser>> DecodeFrom<'ser> for Vec<T> {
    fn decode<'obj>(object: Object<'obj, 'ser>) -> Result<Self, Error> {
        if let Object::List(mut list) = object {
            let mut result = Vec::new();

            while let Some(item) = list.next_object()? {
                result.push(T::decode(item)?);
            }

            Ok(result)
        } else {
            Err(format_err!("Unexpected object type"))
        }
    }
}

impl<'ser> DecodeFrom<'ser> for i64 {
    fn decode<'obj>(object: Object<'obj, 'ser>) -> Result<Self, Error> {
        if let Object::Integer(int) = object {
            Ok(i64::from_str_radix(int, 10)?)
        } else {
            Err(format_err!("Unexpected object type"))
        }
    }
}

impl<'ser> DecodeFrom<'ser> for String {
    fn decode<'obj>(object: Object<'obj, 'ser>) -> Result<Self, Error> {
        if let Object::Bytes(bytes) = object {
            Ok(::std::str::from_utf8(bytes)?.to_owned())
        } else {
            Err(format_err!("Unexpected object type"))
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
struct TestStruct {
    foo: Vec<i64>,
    bar: i64,
}

impl<'ser> DecodeFrom<'ser> for TestStruct {
    #[cfg_attr(feature = "cargo-clippy", allow(blacklisted_name))]
    fn decode<'obj>(object: Object<'obj, 'ser>) -> Result<Self, Error> {
        let mut foo = None;
        let mut bar = None;

        if let Object::Dict(mut dict) = object {
            while let Some((key, value)) = dict.next_pair()? {
                match key {
                    b"foo" => {
                        foo = Some(DecodeFrom::decode(value)?);
                    },
                    b"bar" => {
                        bar = Some(DecodeFrom::decode(value)?);
                    },
                    _ => (), // ignore unknown keys
                }
            }

            Ok(TestStruct {
                foo: foo.ok_or_else(|| format_err!("Missing foo field"))?,
                bar: bar.ok_or_else(|| format_err!("Missing bar field"))?,
            })
        } else {
            Err(format_err!("Expected a dict"))?
        }
    }
}

// test cases for high-level interface
#[test]
fn should_decode_struct() {
    let mut decoder = Decoder::new(SIMPLE_MSG);
    let bencode_object = decoder.next_object().unwrap().unwrap();
    let result = TestStruct::decode(bencode_object).expect("Decoding shouldn't fail");

    assert_eq!(
        result,
        TestStruct {
            foo: vec![2, 3],
            bar: 1,
        }
    )
}
