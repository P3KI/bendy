#[cfg(not(feature = "std"))]
use alloc::{
    collections::{BTreeMap, LinkedList, VecDeque},
    rc::Rc,
    string::String,
    sync::Arc,
    vec::Vec,
};

#[cfg(feature = "std")]
use std::{
    collections::{BTreeMap, HashMap, LinkedList, VecDeque},
    hash::{BuildHasher, Hash},
    rc::Rc,
    sync::Arc,
};

use crate::encoding::{Encoder, Error, SingleItemEncoder};

/// An object that can be encoded into a single bencode object
pub trait ToBencode {
    /// The maximum depth that this object could encode to. Leaves do not consume a level, so an
    /// `i1e` has depth 0 and `li1ee` has depth 1.
    const MAX_DEPTH: usize;

    /// Encode this object into the bencode stream
    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), Error>;

    /// Encode this object to a byte string
    fn to_bencode(&self) -> Result<Vec<u8>, Error> {
        let mut encoder = Encoder::new().with_max_depth(Self::MAX_DEPTH);
        encoder.emit_with(|e| self.encode(e).map_err(Error::into))?;

        let bytes = encoder.get_output()?;
        Ok(bytes)
    }
}

/// Wrapper to allow `Vec<u8>` encoding as bencode string element.
#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct AsString<I>(pub I);

// Forwarding impls
impl<'a, E: 'a + ToBencode + Sized> ToBencode for &'a E {
    const MAX_DEPTH: usize = E::MAX_DEPTH;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), Error> {
        E::encode(self, encoder)
    }
}

#[cfg(feature = "std")]
impl<E: ToBencode> ToBencode for Box<E> {
    const MAX_DEPTH: usize = E::MAX_DEPTH;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), Error> {
        E::encode(&*self, encoder)
    }
}

impl<E: ToBencode> ToBencode for Rc<E> {
    const MAX_DEPTH: usize = E::MAX_DEPTH;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), Error> {
        E::encode(&*self, encoder)
    }
}

impl<E: ToBencode> ToBencode for Arc<E> {
    const MAX_DEPTH: usize = E::MAX_DEPTH;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), Error> {
        E::encode(&*self, encoder)
    }
}

// Base type impls
impl<'a> ToBencode for &'a str {
    const MAX_DEPTH: usize = 0;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), Error> {
        encoder.emit_str(self).map_err(Error::from)
    }
}

impl ToBencode for String {
    const MAX_DEPTH: usize = 0;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), Error> {
        encoder.emit_str(self).map_err(Error::from)
    }
}

macro_rules! impl_encodable_integer {
    ($($type:ty)*) => {$(
        impl ToBencode for $type {
            const MAX_DEPTH: usize = 1;

            fn encode(&self, encoder: SingleItemEncoder) -> Result<(), Error> {
                encoder.emit_int(*self).map_err(Error::from)
            }
        }
    )*}
}

impl_encodable_integer!(u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize);

macro_rules! impl_encodable_iterable {
    ($($type:ident)*) => {$(
        impl <ContentT> ToBencode for $type<ContentT>
        where
            ContentT: ToBencode
        {
            const MAX_DEPTH: usize = ContentT::MAX_DEPTH + 1;

            fn encode(&self, encoder: SingleItemEncoder) -> Result<(), Error> {
                encoder.emit_list(|e| {
                    for item in self {
                        e.emit(item)?;
                    }
                    Ok(())
                })?;

                Ok(())
            }
        }
    )*}
}

impl_encodable_iterable!(Vec VecDeque LinkedList);

impl<'a, ContentT> ToBencode for &'a [ContentT]
where
    ContentT: ToBencode,
{
    const MAX_DEPTH: usize = ContentT::MAX_DEPTH + 1;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), Error> {
        encoder.emit_list(|e| {
            for item in *self {
                e.emit(item)?;
            }
            Ok(())
        })?;

        Ok(())
    }
}

impl<K: AsRef<[u8]>, V: ToBencode> ToBencode for BTreeMap<K, V> {
    const MAX_DEPTH: usize = V::MAX_DEPTH + 1;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), Error> {
        encoder.emit_dict(|mut e| {
            for (k, v) in self {
                e.emit_pair(k.as_ref(), v)?;
            }
            Ok(())
        })?;

        Ok(())
    }
}

#[cfg(feature = "std")]
impl<K, V, S> ToBencode for HashMap<K, V, S>
where
    K: AsRef<[u8]> + Eq + Hash,
    V: ToBencode,
    S: BuildHasher,
{
    const MAX_DEPTH: usize = V::MAX_DEPTH + 1;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), Error> {
        encoder.emit_dict(|mut e| {
            let mut pairs = self
                .iter()
                .map(|(k, v)| (k.as_ref(), v))
                .collect::<Vec<_>>();
            pairs.sort_by_key(|&(k, _)| k);
            for (k, v) in pairs {
                e.emit_pair(k, v)?;
            }
            Ok(())
        })?;

        Ok(())
    }
}

impl<I> ToBencode for AsString<I>
where
    I: AsRef<[u8]>,
{
    const MAX_DEPTH: usize = 1;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), Error> {
        encoder.emit_bytes(self.0.as_ref())?;
        Ok(())
    }
}

impl<I> AsRef<[u8]> for AsString<I>
where
    I: AsRef<[u8]>,
{
    fn as_ref(&self) -> &'_ [u8] {
        self.0.as_ref()
    }
}

impl<'a, I> From<&'a [u8]> for AsString<I>
where
    I: From<&'a [u8]>,
{
    fn from(content: &'a [u8]) -> Self {
        AsString(I::from(content))
    }
}

#[cfg(test)]
mod test {

    #[cfg(not(feature = "std"))]
    use alloc::{borrow::ToOwned, vec};

    use super::*;

    struct Foo {
        bar: u32,
        baz: Vec<String>,
        qux: Vec<u8>,
    }

    impl ToBencode for Foo {
        const MAX_DEPTH: usize = 2;

        fn encode(&self, encoder: SingleItemEncoder) -> Result<(), Error> {
            encoder.emit_dict(|mut e| {
                e.emit_pair(b"bar", &self.bar)?;
                e.emit_pair(b"baz", &self.baz)?;
                e.emit_pair(b"qux", AsString(&self.qux))?;
                Ok(())
            })?;

            Ok(())
        }
    }

    #[test]
    fn simple_encodable_works() {
        let mut encoder = Encoder::new();
        encoder
            .emit(Foo {
                bar: 5,
                baz: vec!["foo".to_owned(), "bar".to_owned()],
                qux: b"qux".to_vec(),
            })
            .unwrap();
        assert_eq!(
            &encoder.get_output().unwrap()[..],
            &b"d3:bari5e3:bazl3:foo3:bare3:qux3:quxe"[..]
        );
    }
}
