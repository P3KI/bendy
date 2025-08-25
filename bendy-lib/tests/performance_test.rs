use bendy::{
    decoding::{Error as DecodingError, FromBencode, Object},
    encoding::{Error as EncodingError, SingleItemEncoder, ToBencode},
};

struct PerformanceTestSubject<T>
where
    T: FromBencode,
{
    list: Vec<Vec<T>>,
}

impl<T> PerformanceTestSubject<T>
where
    T: ToBencode + FromBencode,
{
    fn serialize(&self) -> Vec<u8> {
        self.to_bencode().unwrap()
    }

    fn deserialize(bytes: Vec<u8>) -> Self {
        PerformanceTestSubject::<T>::from_bencode(&bytes).unwrap()
    }
}

impl<T> ToBencode for PerformanceTestSubject<T>
where
    T: ToBencode + FromBencode,
{
    const MAX_DEPTH: usize = 2;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), EncodingError> {
        encoder.emit(&self.list)
        // encoder.emit_list(|e| {
        //     e.emit_int(self.list.len())?;
        //     for vec_elem in &self.list {
        //         for item in vec_elem {
        //             e.emit(item)?;
        //         }
        //     }
        //     Ok(())
        // })
    }
}

impl<T> FromBencode for PerformanceTestSubject<T>
where
    T: FromBencode,
{
    const EXPECTED_RECURSION_DEPTH: usize = 2;

    fn decode_bencode_object(object: Object) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        Ok(PerformanceTestSubject {
            list: Vec::<Vec<T>>::decode_bencode_object(object)?,
        })
        // match object {
        //     Object::List(mut encoded_list) => {
        //         let list_length = match encoded_list.next_object()?.unwrap() {
        //             Object::Integer(i) => Ok(i.parse().unwrap()),
        //             _ => Err(bendy::decoding::Error::unexpected_token(
        //                 "Integer, size of all_nodes Vec",
        //                 "Something else",
        //             )),
        //         }?;
        //         let mut list = Vec::with_capacity(list_length);
        //         for _ in 0..list_length {
        //             list.push(Vec::new());
        //             for _ in 0..list_length {
        //                 list.last_mut().unwrap().push(
        //                     T::decode_bencode_object(encoded_list.next_object()?.unwrap()).unwrap(),
        //                 );
        //             }
        //         }

        //         Ok(PerformanceTestSubject { list })
        //     },
        //     _ => Err(bendy::decoding::Error::unexpected_field("List")),
        // }
    }
}

#[macro_use]
extern crate timeit;

#[test]
fn this_should_take_long() {
    const LIST_SIZE: usize = 1000;
    let test_subject = PerformanceTestSubject::<u32> {
        list: vec![vec![69; LIST_SIZE]; LIST_SIZE],
    };
    timeit!({
        test_subject.serialize();
    });
    let serialized = test_subject.serialize();
    timeit!({
        PerformanceTestSubject::<u32>::deserialize(serialized.clone());
    });
    // let _deserialized = PerformanceTestSubject::<u32>::deserialize(serialized);
}
