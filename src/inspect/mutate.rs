use core::ops::RangeBounds;
use core::slice::SliceIndex;

use crate::inspect::*;

impl<'ser, 'obj, 'otherser, 'min> Inspectable<'ser>
where
    'otherser: 'ser,
{
    // Replaces one Inspectable with another. The replacement
    // must have a lifetime at least as long as the one it is
    // replacing.
    pub fn replace(&'obj mut self, other: Inspectable<'otherser>) {
        *self = other;
    }
}

impl<'ser, 'obj> Inspectable<'ser>
{
    pub fn clone_and_mutate<F>(&'obj self, path: &crate::inspect::traverse::PathBuilder, f: F) -> Inspectable<'ser>
        where F: FnOnce(&mut Inspectable<'ser>)
    {
        let mut c = self.clone();
        c.mutate(path, f);
        c
    }

    pub fn mutate<F>(&'obj mut self, path: &crate::inspect::traverse::PathBuilder, f: F)
        where F: FnOnce(&mut Inspectable<'ser>)
    {
        self.find_mut(path).apply(f)
    }

    pub fn apply<F>(&'obj mut self, f: F)
        where F: FnOnce(&mut Inspectable<'ser>)
    {
        f(self);
    }

    pub fn remove_nth(&'obj mut self, idx: usize) {
        match self {
            Inspectable::Dict(x) => x.remove_nth(idx),
            Inspectable::List(x) => x.remove_nth(idx),
            _ => panic!("Remove Nth mutation only available on Dicts and Lists")
        }
    }

    pub fn truncate(&'obj mut self) {
        match self {
            Inspectable::String(instring) => instring.truncate(),
            _ => panic!("Truncate mutation only available on ByteStrings")
        }
    }

    pub fn set_content_byterange<R>(&'obj mut self, range: R, byte: u8)
        where R: RangeBounds<usize>,
    {
        match self {
            Inspectable::String(instring) => instring.set_content_byterange(range, byte),
            _ => panic!("Set Content ByteRange mutation only available on ByteStrings")
        }
    }

    pub fn clear_content(&'obj mut self) {
        match self {
            Inspectable::String(x) => x.clear_content(),
            Inspectable::Int(x) => x.clear_content(),
            Inspectable::Raw(x) => x.to_mut().clear(),
            Inspectable::Dict(x) => x.clear_content(),
            Inspectable::List(x) => x.clear_content(),
        }
    }
}

impl<'ser, 'obj, 'other> Inspectable<'ser> {
    /// Removes all entries in the dict with the given key.
    /// Note that it's normally an error for a bencode dict to have
    /// multiple identical keys.
    ///
    /// Assumes that all entries' keys are Byte Strings, which is
    /// also required for valid bencode.
    pub fn remove_entry(&'obj mut self, dict_key: &'other [u8]) {
        match self {
            Inspectable::Dict(x) => x.remove_entry(dict_key),
            _ => panic!("Remove Entry mutation only available on Dicts")
        }
    }
}

impl<'ser, 'obj> InInt<'ser> {
    /// Sets the InInt to a specified i64 value.
    /// Allocates a String.
    pub fn set(&'obj mut self, value: i64) {
        *self.bytes.to_mut() = value.to_string();
    }

    pub fn clear_content(&'obj mut self) {
        self.set(0);
    }
}

impl<'ser, 'obj> InDict<'ser> {
    /// Sets the InInt to a specified i64 value.
    /// Allocates a String.
    pub fn clear_content(&'obj mut self) {
        self.items.clear();
    }

    pub fn remove_nth(&'obj mut self, idx: usize) {
        self.items.remove(idx);
    }

}

impl<'ser, 'obj, 'other> InDict<'ser> {
    /// Removes all entries in the dict with the given key.
    /// Note that it's normally an error for a bencode dict to have
    /// multiple identical keys.
    ///
    /// Assumes that all entries' keys are Byte Strings, which is
    /// also required for valid bencode.
    pub fn remove_entry(&'obj mut self, dict_key: &'other [u8]) {
        self.items.retain(|entry| entry.key.string().bytes != dict_key);
    }
}

impl<'ser, 'obj> InList<'ser> {
    pub fn clear_content(&'obj mut self) {
        self.items.clear();
    }

    pub fn remove_nth(&'obj mut self, idx: usize) {
        self.items.remove(idx);
    }
}

impl<'ser, 'obj> InString<'ser> {
    pub fn set_fake_length(&'obj mut self, length: usize) {
        self.fake_length = Some(length);
    }

    pub fn clear_fake_length(&'obj mut self) {
        self.fake_length = None;
    }

    pub fn set_content_string(&'obj mut self, other: String) {
        self.bytes = Cow::Owned(other.into());
    }

    pub fn set_content_vec(&'obj mut self, other: Vec<u8>) {
        self.bytes = Cow::Owned(other);
    }

    pub fn truncate(&'obj mut self) {
        let bytes = self.bytes.to_mut();
        let len = bytes.len();
        let new_len = len/2;
        if len <= new_len {
            bytes.clear();
            return;
        }
        bytes.truncate(new_len);
    }

    pub fn clear_content(&'obj mut self) {
        self.bytes.to_mut().clear();
    }

    pub fn set_content_byterange<R>(&'obj mut self, range: R, byte: u8)
        where R: RangeBounds<usize>,
    {
        let len = self.bytes.len();
        let (start, stop) = normalize_range(range, len);
        let bytes = self.bytes.to_mut();
        for i in start..stop {
            let mref = bytes.get_mut(i)
                .expect("Tried to replace byte outside of ByteString");
            *mref = byte;
        }
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

fn normalize_range<R>(range: R, max_end: usize) -> (usize, usize)
    where R: RangeBounds<usize>
{
    // Unstable for now
    // if range.is_empty() {
    //     panic!("Can't set byterange contents with an empty range");
    // }

    let beg = range.start_bound();
    let end = range.end_bound();

    let start = match beg {
        core::ops::Bound::Included(n) => *n,
        core::ops::Bound::Excluded(n) => n+1, // Probably doesn't matter?
        core::ops::Bound::Unbounded => 0,
    };

    let stop = match end {
        core::ops::Bound::Included(n) => *n+1,
        core::ops::Bound::Excluded(n) => *n,
        core::ops::Bound::Unbounded => max_end,
    };

    if start >= stop {
        panic!("Can't set byterange contents with an empty range");
    }

    (start, stop)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_byterange_test() {
        let mut i = inspect(b"5:AAAAA");
        assert_eq!(5, i.string().bytes.len());

        i.string_mut().set_content_byterange(.., b'0');
        assert_eq!(b"5:00000", i.emit().as_slice());

        let mut i = inspect(b"5:AAAAA");
        i.string_mut().set_content_byterange(0.., b'0');
        assert_eq!(b"5:00000", i.emit().as_slice());

        let mut i = inspect(b"5:AAAAA");
        i.string_mut().set_content_byterange(0..=4, b'0');
        assert_eq!(b"5:00000", i.emit().as_slice());

        let mut i = inspect(b"5:AAAAA");
        i.string_mut().set_content_byterange(0..5, b'0');
        assert_eq!(b"5:00000", i.emit().as_slice());

        let mut i = inspect(b"5:AAAAA");
        i.string_mut().set_content_byterange(0..=1, b'0');
        assert_eq!(b"5:00AAA", i.emit().as_slice());

        let mut i = inspect(b"5:AAAAA");
        i.string_mut().set_content_byterange(0..2, b'0');
        assert_eq!(b"5:00AAA", i.emit().as_slice());

        let mut i = inspect(b"5:AAAAA");
        i.string_mut().set_content_byterange(0..3, b'0');
        assert_eq!(b"5:000AA", i.emit().as_slice());

        let mut i = inspect(b"5:AAAAA");
        i.string_mut().set_content_byterange(0..4, b'0');
        assert_eq!(b"5:0000A", i.emit().as_slice());

        let mut i = inspect(b"5:AAAAA");
        i.string_mut().set_content_byterange(2..4, b'0');
        assert_eq!(b"5:AA00A", i.emit().as_slice());

        let mut i = inspect(b"5:AAAAA");
        i.string_mut().set_content_byterange(2..5, b'0');
        assert_eq!(b"5:AA000", i.emit().as_slice());

    }

    #[test]
    #[should_panic]
    fn out_of_bounds_range_inclusive() {
        let mut i = inspect(b"5:AAAAA");
        i.string_mut().set_content_byterange(2..=5, b'0');
    }

    #[test]
    #[should_panic]
    fn out_of_bounds_range() {
        let mut i = inspect(b"5:AAAAA");
        i.string_mut().set_content_byterange(2..6, b'0');
    }

    #[test]
    #[should_panic]
    fn empty_range() {
        let mut i = inspect(b"5:AAAAA");
        i.string_mut().set_content_byterange(5..1, b'0');
    }

}
