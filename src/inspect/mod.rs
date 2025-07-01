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

pub mod display;
pub mod mutate;
pub mod parse;
pub mod traverse;

/// Attempt to decode a u8 buffer into an inspectable.
/// Panics on error. Use Inspectable::try_from for a
/// fallible alternative.
pub fn inspect<'ser>(buf: &'ser [u8]) -> Inspectable<'ser> {
    Inspectable::try_from(buf).expect("Could not decode buffer into inspectable")
}

/// Builds a path that can be used to traverse from an
/// Inspectable to a child, grandchild, etc Inspectable.
pub fn inspect_path<'ser>() -> traverse::PathBuilder<'ser> {
    traverse::PathBuilder::new()
}

/// An object that represents something that bencode can
/// decode to. Usable for reflection, modification,
/// testing, and pretty printing.
///
/// Use in production code not recommended.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Inspectable<'ser> {
    Int(InInt<'ser>),
    String(InString<'ser>),
    Raw(Cow<'ser, [u8]>),
    Dict(InDict<'ser>),
    List(InList<'ser>),
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
                    for InDictEntry { key, value } in x.items.iter() {
                        dispatch(key, out);
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InString<'ser> {
    pub bytes: Cow<'ser, [u8]>,
    pub fake_length: Option<usize>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InInt<'ser> {
    pub bytes: Cow<'ser, str>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InDictEntry<'ser> {
    /// This one must be an Inspectable::String
    /// for the bencode to be valid
    pub key: Inspectable<'ser>,
    pub value: Inspectable<'ser>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InDict<'ser> {
    pub items: Vec<InDictEntry<'ser>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InList<'ser> {
    pub items: Vec<Inspectable<'ser>>,
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
        self.bytes.parse().expect("Could not parse InInt as i64")
    }
}

impl<'ser> InString<'ser> {
    pub fn new(buf: &'ser [u8]) -> InString<'ser> {
        InString {
            bytes: Cow::from(buf),
            fake_length: None,
        }
    }

    /// Returns the number of bytes in the bytestring,
    /// or the fake length of the bytestring if set.
    pub fn len(&self) -> usize {
        self.fake_length.unwrap_or_else(|| self.bytes.len())
    }

    pub fn content(&self) -> &[u8] {
        &*self.bytes
    }
}

impl<'obj, 'ser> InList<'ser> {
    pub fn new() -> Self {
        InList { items: Vec::new() }
    }
}

impl<'obj, 'ser> InDict<'ser> {
    pub fn new() -> Self {
        InDict { items: Vec::new() }
    }
}

impl<'ser> InDictEntry<'ser> {
    pub fn new(key: InString<'ser>, value: Inspectable<'ser>) -> Self {
        InDictEntry {
            key: Inspectable::String(key),
            value,
        }
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
        assert_eq!(b"one".as_slice(), &*t0.key.string().bytes);
        assert_eq!(b"two".as_slice(), &*t1.key.string().bytes);
        assert_eq!(b"zzzzz".as_slice(), &*t2.key.string().bytes);
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
        i.list_mut()
            .nth_mut(1)
            .dict_mut()
            .nth_mut(0)
            .value
            .int_mut()
            .set(1);
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
        i.list_mut()
            .nth_mut(1)
            .dict_mut()
            .nth_mut(0)
            .key
            .string_mut()
            .set_fake_length(0);
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
        i.list_mut()
            .nth_mut(0)
            .string_mut()
            .set_content_string("one".to_string());
        i.list_mut().nth_mut(1).string_mut().set_content_str("two");
        let entry = i.list_mut().nth_mut(2).dict_mut().nth_mut(0);
        entry.key.string_mut().set_content_u8(b"three");
        entry.value.string_mut().set_content_vec(Vec::from(b"four"));
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
