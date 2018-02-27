extern crate bencode_zero;

use bencode_zero::decoder::*;

static SIMPLE_MSG: &'static [u8] = b"d3:bari1e3:fooli2ei3eee";

// test of high-level interface
// Normally, a trait like this would have a better error type
use std::error::Error as StdError;
trait DecodeFrom<'ser>: Sized {
    fn decode<'obj>(object: Object<'obj, 'ser>) -> Result<Self, Box<StdError>>;
}

impl<'ser, T: DecodeFrom<'ser>> DecodeFrom<'ser> for Vec<T> {
    fn decode<'obj>(object: Object<'obj, 'ser>) -> Result<Self, Box<StdError>> {
        match object {
            Object::List(mut list) => {
                let mut result = Vec::new();
                while let Some(item) = list.next()? {
                    result.push(T::decode(item)?);
                }
                Ok(result)
            }
            _ => Err("Unexpected object type")?
        }
    }
}

impl<'ser> DecodeFrom<'ser> for i64 {
    fn decode<'obj>(object: Object<'obj, 'ser>) -> Result<Self, Box<StdError>> {
        match object {
            Object::Integer(int) => {
                Ok(i64::from_str_radix(int, 10)?)
            }
            _ => Err("Unexpected object type")?
        }
    }
}

impl<'ser> DecodeFrom<'ser> for String {
    fn decode<'obj>(object: Object<'obj, 'ser>) -> Result<Self, Box<StdError>> {
        if let Object::Bytes(bytes) = object {
            Ok(::std::str::from_utf8(bytes)?.to_owned())
        } else {
            Err("Unexpected object type")?
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
struct TestStruct {
    foo: Vec<i64>,
    bar: i64,
}

impl<'ser> DecodeFrom<'ser> for TestStruct {
    fn decode<'obj>(object: Object<'obj, 'ser>) -> Result<Self, Box<StdError>> {
        let mut foo = None;
        let mut bar = None;

        if let Object::Dict(mut dict) = object {
            while let Some((key, value)) = dict.next()? {
                match key {
                    b"foo" => { foo = Some(DecodeFrom::decode(value)?); }
                    b"bar" => { bar = Some(DecodeFrom::decode(value)?); }
                    _ => (), // ignore unknown keys
                }
            }

            Ok(TestStruct{
                foo: foo.ok_or("Missing foo field")?,
                bar: bar.ok_or("Missing bar field")? })
        } else {
            Err("Expected a dict")?
        }
    }
}

// test cases for high-level interface
#[test]
fn should_decode_struct() {
    let mut decoder = Decoder::new(SIMPLE_MSG);
    let result = TestStruct::decode(decoder.next().unwrap().unwrap())
        .expect("Decoding shouldn't fail");
    assert_eq!(result, TestStruct{
        foo: vec![2,3],
        bar: 1,
    })
}
