/// Provides facilities for creating an AST-like object
/// tree from any valid bencode. Decoding in this way
/// is done using recursion, so stack size limits apply.
///
/// Use for reflection, modifying, testing, and pretty printing.
///
/// Not recommended for use in production code.

#[cfg(not(feature = "std"))]
use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec::Vec,
};
#[cfg(feature = "std")]
use std::{borrow::Cow, vec::Vec};

use crate::{
    decoding::{Decoder, DictDecoder, Error as DecodeError, ListDecoder, Object},
    state_tracker::StructureError,
};

pub mod display;

/// Attempt to decode a u8 buffer into an inspectable.
/// Panics on error. Use Inspectable::try_from for a
/// fallible alternative.
pub fn inspect(buf: &[u8]) -> Inspectable {
    Inspectable::try_from(buf)
        .expect("Could not decode buffer into inspectable")
}

/// An object that represents something that bencode can
/// decode to. Usable for reflection, modification,
/// testing, and pretty printing.
///
/// Use in production code not recommended.
#[derive(Debug, PartialEq, Eq)]
pub enum Inspectable<'ser> {
    Int(InInt<'ser>),
    String(InString<'ser>),
    Raw(Cow<'ser, [u8]>),
    Dict(InDict<'ser>),
    List(InList<'ser>),
}

impl<'ser> TryFrom<&'ser [u8]> for Inspectable<'ser> {
    type Error = DecodeError;

    /// Decodes bencode data into an Inspectable
    fn try_from(buf: &'ser [u8]) -> Result<Inspectable<'ser>, Self::Error> {
        let mut decoder = Decoder::new(buf);
        let obj = decoder
            .next_object()?
            .ok_or_else(|| StructureError::UnexpectedEof)?;
        Self::try_from(obj)
    }
}

impl<'ser> Inspectable<'ser> {
    pub fn new_raw(buf: &'ser [u8]) -> Inspectable<'ser> {
        Inspectable::Raw(Cow::from(buf))
    }

    pub fn new_int(i: i64) -> Inspectable<'ser> {
        let inint = InInt {
            bytes: Cow::Owned(i.to_string()),
        };
        Inspectable::Int(inint)
    }

    pub fn new_string(buf: &'ser [u8]) -> Inspectable<'ser> {
        Inspectable::String(InString::new(buf))
    }

    pub fn new_dict() -> Inspectable<'ser> {
        Inspectable::Dict(InDict::new())
    }

    pub fn new_list() -> Inspectable<'ser> {
        Inspectable::List(InList::new())
    }

    pub fn emit(&self) -> Vec<u8> {
        let mut res = Vec::new();

        fn emit_str(s: &InString, out: &mut Vec<u8>) {
            let len = s.len().to_string();
            out.extend_from_slice(len.as_bytes());
            out.push(b':');
            out.extend_from_slice(s.content())
        }

        fn dispatch(i: &Inspectable, out: &mut Vec<u8>) {
            match i {
                Inspectable::String(x) => {
                    emit_str(x, out);
                },
                Inspectable::Raw(x) => {
                    out.extend_from_slice(&*x);
                },
                Inspectable::Int(x) => {
                    out.push(b'i');
                    out.extend_from_slice(x.bytes.as_bytes());
                    out.push(b'e');
                },
                Inspectable::List(x) => {
                    out.push(b'l');
                    for item in x.items.iter() {
                        dispatch(item, out);
                    }
                    out.push(b'e');
                },
                Inspectable::Dict(x) => {
                    out.push(b'd');
                    for InTuple { key, value } in x.items.iter() {
                        emit_str(key, out);
                        dispatch(value, out);
                    }
                    out.push(b'e');
                },
            }
        }
        dispatch(self, &mut res);
        res
    }
}

impl<'ser, 'other, 'min> Inspectable<'ser>
where
    'other: 'ser,
{
    // Replaces one Inspectable with another. The replacement
    // must have a lifetime at least as long as the one it is
    // replacing.
    pub fn replace(&mut self, other: Inspectable<'other>) {
        *self = other;
    }
}

macro_rules! variant_accessors {
    ($(($name:ident, $mutname:ident, $source:ident, $target:ty))*) => {$(
        impl<'obj, 'ser> Inspectable<'ser> {
            pub fn $name(&'obj self) -> &'obj $target {
                match self {
                    Inspectable::$source(x) => x,
                    _ => panic!("Attempted to take non-{} Inspectable as {}", stringify!($source), stringify!($target))
                }
            }
            pub fn $mutname(&'obj mut self) -> &'obj mut $target {
                match self {
                    Inspectable::$source(x) => x,
                    _ => panic!("Attempted to take non-{} Inspectable as mut_{}", stringify!($source), stringify!($target))
                }
            }
        }
    )*};
}

variant_accessors! {
    (string, string_mut, String, InString<'ser>)
    (int, int_mut, Int, InInt<'ser>)
    (raw, raw_mut, Raw, Cow<'ser, [u8]>)
    (dict, dict_mut, Dict, InDict<'ser>)
    (list, list_mut, List, InList<'ser>)
}

#[derive(Debug, PartialEq, Eq)]
pub struct InString<'ser> {
    bytes: Cow<'ser, [u8]>,
    fake_length: Option<usize>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InInt<'ser> {
    bytes: Cow<'ser, str>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InTuple<'ser> {
    key: InString<'ser>,
    value: Inspectable<'ser>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InDict<'ser> {
    items: Vec<InTuple<'ser>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InList<'ser> {
    items: Vec<Inspectable<'ser>>,
}

impl<'ser> InInt<'ser> {
    pub fn new(buf: &'ser str) -> Self {
        InInt {
            bytes: Cow::from(buf),
        }
    }

    /// Returns an i64 of the value represented.
    /// Panics if this is not possible.
    #[must_use]
    pub fn as_i64(&self) -> i64 {
        self.bytes.parse()
            .expect("Could not parse InInt as i64")
    }

    /// Sets the InInt to a specified i64 value.
    /// Allocates a String.
    pub fn set(&mut self, value: i64) {
        *self.bytes.to_mut() = value.to_string();
    }
}

impl<'ser> InString<'ser> {
    pub fn new(buf: &'ser [u8]) -> InString<'ser> {
        InString {
            bytes: Cow::from(buf),
            fake_length: None,
        }
    }

    pub fn len(&self) -> usize {
        self.fake_length.unwrap_or_else(|| self.bytes.len())
    }

    pub fn content(&self) -> &[u8] {
        &*self.bytes
    }

    pub fn set_fake_length(&mut self, length: usize) {
        self.fake_length = Some(length);
    }

    pub fn clear_fake_length(&mut self) {
        self.fake_length = None;
    }

    pub fn set_content_string(&mut self, other: String) {
        self.bytes = Cow::Owned(other.into());
    }

    pub fn set_content_vec(&mut self, other: Vec<u8>) {
        self.bytes = Cow::Owned(other);
    }
}

impl<'me, 'other, 'ser> InString<'ser>
where
    'other: 'ser,
{
    pub fn set_content_str(&'me mut self, other: &'other str) {
        self.bytes = Cow::from(other.as_bytes());
    }

    pub fn set_content_u8(&'me mut self, other: &'other [u8]) {
        self.bytes = Cow::from(other);
    }
}

impl<'obj, 'ser> InList<'ser> {
    pub fn new() -> Self {
        InList { items: Vec::new() }
    }

    pub fn nth(&'obj self, idx: usize) -> &'obj Inspectable<'ser> {
        self.items.get(idx).expect("Could not access nth Inspectable in list")
    }

    pub fn nth_mut(&'obj mut self, idx: usize) -> &'obj mut Inspectable<'ser> {
        self.items.get_mut(idx).expect("Could not mutably access nth Inspectable in list")
    }
}

impl<'obj, 'ser> InDict<'ser> {
    pub fn new() -> Self {
        InDict { items: Vec::new() }
    }

    pub fn nth(&'obj self, idx: usize) -> &'obj InTuple<'ser> {
        self.items.get(idx).expect("Could not access nth Inspectable in list")
    }

    pub fn nth_mut(&'obj mut self, idx: usize) -> &'obj mut InTuple<'ser> {
        self.items.get_mut(idx).expect("Could not mutably access nth Inspectable in list")
    }
}

impl<'obj, 'ser, 'other> InDict<'ser> {
    pub fn entry(&'obj self, name: &'other [u8]) -> &'obj InTuple<'ser> {
        self.items.iter().find(|InTuple{key, ..}| key.bytes == name)
            .expect("Could not find a tuple with requested key")
    }

    pub fn entry_mut(&'obj mut self, name: &'other [u8]) -> &'obj mut InTuple<'ser> {
        self.items.iter_mut().find(|InTuple{key, ..}| key.bytes == name)
            .expect("Could not find a tuple with requested key")
    }
}

impl<'ser> InTuple<'ser> {
    pub fn new(key: InString<'ser>, value: Inspectable<'ser>) -> Self {
        InTuple { key, value }
    }
}

impl<'obj, 'ser> TryFrom<Object<'obj, 'ser>> for Inspectable<'ser> {
    type Error = DecodeError;

    fn try_from(object: Object<'obj, 'ser>) -> Result<Inspectable<'ser>, DecodeError> {
        Ok(match object {
            Object::List(ld) => Inspectable::List(InList::try_from(ld)?),
            Object::Dict(dd) => Inspectable::Dict(InDict::try_from(dd)?),
            Object::Integer(i) => Inspectable::Int(InInt::new(i)),
            Object::Bytes(b) => Inspectable::String(InString::new(b)),
        })
    }
}

impl<'obj, 'ser> TryFrom<ListDecoder<'obj, 'ser>> for InList<'ser> {
    type Error = DecodeError;

    fn try_from(mut ld: ListDecoder<'obj, 'ser>) -> Result<Self, Self::Error> {
        let mut items: Vec<Inspectable<'ser>> = Vec::new();
        while let Some(item) = ld.next_object()? {
            items.push(item.try_into()?);
        }

        Ok(InList { items })
    }
}
impl<'obj, 'ser> TryFrom<DictDecoder<'obj, 'ser>> for InDict<'ser> {
    type Error = DecodeError;

    fn try_from(mut dd: DictDecoder<'obj, 'ser>) -> Result<Self, Self::Error> {
        let mut items: Vec<InTuple> = Vec::new();

        while let Some((k, v)) = dd.next_pair()? {
            items.push(InTuple {
                key: InString::new(k),
                value: v.try_into()?,
            })
        }

        Ok(InDict { items })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_traversal_test() {
        let buf = b"\
        l\
            i99e\
            5:hello\
            d\
                3:one\
                i11e\
                3:two\
                i22e\
                5:zzzzz\
                i33e\
            e\
        e";
        let i = inspect(buf);
        let l = i.list();
        assert_eq!(3, l.items.len());
        assert_eq!(99, l.nth(0).int().as_i64());
        assert_eq!(b"hello".as_slice(), &*l.nth(1).string().bytes);
        let d = i.list().nth(2).dict();
        let t0 = d.nth(0);
        let t1 = d.nth(1);
        let t2 = d.entry(b"zzzzz");
        assert_eq!(b"one".as_slice(), &*t0.key.bytes);
        assert_eq!(b"two".as_slice(), &*t1.key.bytes);
        assert_eq!(b"zzzzz".as_slice(), &*t2.key.bytes);
        assert_eq!(11, t0.value.int().as_i64());
        assert_eq!(22, t1.value.int().as_i64());
        assert_eq!(33, t2.value.int().as_i64());
    }

    #[test]
    fn int_modify_test() {
        let buf = b"i64e";
        let mut i = inspect(buf);
        assert_eq!(64, i.int().as_i64());
        i.int_mut().set(32);
        assert_eq!(32, i.int().as_i64());
        assert_eq!("i32e", i.to_string().as_str());
        assert_eq!(b"i32e", i.emit().as_slice());

        let buf = b"\
        l\
            i43770e\
            d\
                3:one\
                i11e\
            e\
        e";
        let mut i = inspect(buf);
        i.list_mut().nth_mut(0).int_mut().set(2);
        i.list_mut().nth_mut(1).dict_mut().nth_mut(0).value.int_mut().set(1);
        assert_eq!(b"li2ed3:onei1eee".as_slice(), i.emit().as_slice());
    }

    #[test]
    fn fake_bytestring_length_test() {
        let buf = b"\
        l\
            5:hello\
            d\
                3:one\
                i11e\
            e\
        e";
        let mut i = inspect(buf);
        i.list_mut().nth_mut(0).string_mut().set_fake_length(20);
        i.list_mut().nth_mut(1).dict_mut().nth_mut(0).key.set_fake_length(0);
        assert_eq!(b"l20:hellod0:onei11eee".as_slice(), i.emit().as_slice());
        assert_eq!("l20:hellod0:onei11eee", i.to_string().as_str());
    }

    #[test]
    fn bytestring_set_content_test() {
        let buf = b"\
        l\
            5:hello\
            5:world\
            d\
                5:hello\
                5:world\
            e\
        e";
        let mut i = inspect(buf);
        i.list_mut().nth_mut(0).string_mut().set_content_string("one".to_string());
        i.list_mut().nth_mut(1).string_mut().set_content_str("two");
        let tuple = i.list_mut().nth_mut(2).dict_mut().nth_mut(0);
        tuple.key.set_content_u8(b"three");
        tuple.value.string_mut().set_content_vec(Vec::from(b"four"));
        assert_eq!(
            b"l3:one3:twod5:three4:fouree".as_slice(),
            i.emit().as_slice()
        );
        assert_eq!("l3:one3:twod5:three4:fouree", i.to_string().as_str());

        let mut i = inspect(b"5:hello");
        i.string_mut().set_content_u8(b"\x00\x01");
        assert_eq!(b"2:\x00\x01".as_slice(), i.emit().as_slice());
        assert_eq!("2:\\x00\\x01", i.to_string().as_str());
    }

    #[test]
    fn replace_test() {
        let buf = b"\
        l\
            i99e\
            5:hello\
            d\
                3:one\
                i11e\
                3:two\
                i22e\
            e\
        e";
        let mut i = inspect(buf);
        let l = i.list_mut();
        l.nth_mut(0).replace(Inspectable::new_string(b"aaa"));
        l.nth_mut(1).replace(inspect(b"li1ei2ei3ee"));
        l.nth_mut(2).replace(Inspectable::new_int(5));
        assert_eq!(b"l3:aaali1ei2ei3eei5ee".as_slice(), i.emit().as_slice());
    }
}
