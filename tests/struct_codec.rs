use bendy::{
    decoding::{Error as DecodingError, FromBencode, Object},
    encoding::{Error as EncodingError, SingleItemEncoder, ToBencode},
};

#[derive(PartialEq, Eq, Debug)]
struct Example {
    foo: Vec<i64>,
    bar: i64,
}

impl ToBencode for Example {
    const MAX_DEPTH: usize = 2;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), EncodingError> {
        encoder.emit_dict(|mut dict| {
            dict.emit_pair(b"bar", &self.bar)?;
            dict.emit_pair(b"foo", &self.foo)
        })
    }
}

impl FromBencode for Example {
    const EXPECTED_RECURSION_DEPTH: usize = 2;

    fn decode_bencode_object(object: Object) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let mut foo = None;
        let mut bar = None;

        let mut dict = object.try_into_dictionary()?;
        while let Some((key, value)) = dict.next_pair()? {
            match key {
                b"foo" => {
                    foo = Vec::decode_bencode_object(value).map(Some)?;
                },
                b"bar" => {
                    bar = i64::decode_bencode_object(value).map(Some)?;
                },
                _ => (), // ignore unknown keys
            }
        }

        Ok(Example {
            foo: foo.ok_or_else(|| DecodingError::missing_field("foo"))?,
            bar: bar.ok_or_else(|| DecodingError::missing_field("bar"))?,
        })
    }
}

#[test]
fn should_encode_struct() {
    let example = Example {
        foo: vec![2, 3],
        bar: 1,
    };
    let encoded = example.to_bencode().expect("example encoding is broken");

    assert_eq!(encoded, b"d3:bari1e3:fooli2ei3eee".to_vec(),)
}

#[test]
fn should_decode_struct() {
    let encoded = b"d3:bari1e3:fooli2ei3eee".to_vec();
    let example = Example::from_bencode(&encoded).expect("example decoding is broken");

    assert_eq!(
        example,
        Example {
            foo: vec![2, 3],
            bar: 1,
        }
    )
}

#[test]
fn validate_round_trip() {
    let example = Example {
        foo: vec![2, 3],
        bar: 1,
    };

    let encoded = example.to_bencode().expect("example encoding is broken");
    let decoded = Example::from_bencode(&encoded).expect("example decoding is broken");

    assert_eq!(example, decoded);
}
