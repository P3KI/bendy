//! Methods for mutating [`Inspectable`]s.
//!
//! Refer to the [documentation here][crate::inspect#mutating].

use core::ops::RangeBounds;

use crate::inspect::*;

impl<'ser, 'obj, 'otherser> Inspectable<'ser>
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

impl<'ser, 'obj> Inspectable<'ser> {
    /// Clones an Inspectable AST and applies the given function to
    /// the node indicated by the given Path on the resulting clone.
    ///
    /// Panics if the Path fails to find a node.
    #[must_use]
    pub fn clone_and_mutate<F>(
        &'obj self,
        path: &crate::inspect::traverse::PathBuilder,
        f: F,
    ) -> Inspectable<'ser>
    where
        F: FnOnce(&mut Inspectable<'ser>),
    {
        let mut c = self.clone();
        c.mutate(path, f);
        c
    }

    /// Applies the given function to the node in the Inspectable AST indicated by the given Path.
    ///
    /// Panics if the Path fails to find a node.
    pub fn mutate<F>(&'obj mut self, path: &crate::inspect::traverse::PathBuilder, f: F)
    where
        F: FnOnce(&mut Inspectable<'ser>),
    {
        self.find(path).apply(f)
    }

    /// Applies the given function to the Inspectable
    pub fn apply<F>(&'obj mut self, f: F)
    where
        F: FnOnce(&mut Inspectable<'ser>),
    {
        f(self);
    }

    /// Remove the nth list item or dict entry.
    ///
    /// Panics if this is not an [`Inspectable::Dict`] or [`Inspectable::List`].
    pub fn remove_nth(&'obj mut self, idx: usize) {
        match self {
            Inspectable::Dict(x) => x.remove_nth(idx),
            Inspectable::List(x) => x.remove_nth(idx),
            _ => panic!("Remove Nth mutation only available on Dicts and Lists"),
        }
    }

    /// Attempts an [`InString::truncate`] operation on the Inspectable.
    ///
    /// Panics if this is not an [`Inspectable::String`].
    pub fn truncate(&'obj mut self) {
        match self {
            Inspectable::String(instring) => instring.truncate(),
            _ => panic!("Truncate mutation only available on ByteStrings"),
        }
    }

    /// Attempts an [`InString::set_content_byterange`] operation on the Inspectable.
    ///
    /// Panics if this is not an [`Inspectable::String`].
    pub fn set_content_byterange<R>(&'obj mut self, range: R, byte: u8)
    where
        R: RangeBounds<usize>,
    {
        match self {
            Inspectable::String(instring) => instring.set_content_byterange(range, byte),
            _ => panic!("Set Content ByteRange mutation only available on ByteStrings"),
        }
    }

    /// Applies one of the following operations on the Inspectable, depending on its type:
    /// * [`InString::clear_content`]
    /// * [`InInt::clear_content`]
    /// * [`Vec::clear`] for [`Inspectable::Raw`]
    /// * [`InDict::clear_content`]
    /// * [`InList::clear_content`]
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
            _ => panic!("Remove Entry mutation only available on Dicts"),
        }
    }
}

impl<'ser, 'obj> InInt<'ser> {
    /// Sets the InInt to a specified i64 value.
    /// Allocates a String.
    pub fn set(&'obj mut self, value: i64) {
        *self.bytes.to_mut() = value.to_string();
    }

    /// Set the integer to 0.
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

    /// Remove the nth entry in the dict.
    pub fn remove_nth(&'obj mut self, idx: usize) {
        self.items.remove(idx);
    }

    /// Sorts the dictionary's entries by key, so that valid bencode is
    /// produced (assuming no other problems, like duplicate keys.)
    ///
    /// Assumes all entries' keys are byte strings ([`Inspectable::String`]
    /// objects). Panics otherwise.
    ///
    /// # Example
    /// ```
    /// # use bendy::inspect::*;
    /// let mut d = Inspectable::new_dict();
    /// d.dict().items.push(InDictEntry::new(
    ///     Inspectable::new_string(b"zzz"),
    ///     Inspectable::new_int(1),
    /// ));
    /// d.dict().items.push(InDictEntry::new(
    ///     Inspectable::new_string(b"aaa"),
    ///     Inspectable::new_int(0),
    /// ));
    /// d.dict().sort();
    /// assert_eq!(&d.emit(), b"d3:aaai0e3:zzzi1ee");
    /// ```
    pub fn sort(&'obj mut self) {
        self.items.sort_unstable_by(|a, b| {
            let a = &*a.key.string_ref().bytes;
            let b = &*b.key.string_ref().bytes;
            a.cmp(b)
        });
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
        self.items
            .retain(|entry| entry.key.string_ref().bytes != dict_key);
    }
}

impl<'ser, 'obj> InList<'ser> {
    /// Empties the list.
    pub fn clear_content(&'obj mut self) {
        self.items.clear();
    }

    /// Removes the nth item in the list.
    pub fn remove_nth(&'obj mut self, idx: usize) {
        self.items.remove(idx);
    }
}

impl<'ser, 'obj> InString<'ser> {
    /// Set a fake length for the bytestring. This will change
    /// its length prefix when emitted as bencode.
    pub fn set_fake_length(&'obj mut self, length: usize) {
        self.fake_length = Some(length);
    }

    /// Unset the fake length. This will cause the length
    /// prefix to be correctly calculated and used when
    /// emitted as bencode.
    pub fn clear_fake_length(&'obj mut self) {
        self.fake_length = None;
    }

    /// Change the contents of the bytestring. Takes a string
    /// which is coerced into a [`Vec<u8>`].
    pub fn set_content_string(&'obj mut self, other: String) {
        self.bytes = Cow::Owned(other.into());
    }

    /// Change the contents of the bytestring.
    pub fn set_content_vec(&'obj mut self, other: Vec<u8>) {
        self.bytes = Cow::Owned(other);
    }

    /// Truncates the bytestring to half its size. Empties it
    /// if the bytestring is too short to be truncated.
    pub fn truncate(&'obj mut self) {
        let bytes = self.bytes.to_mut();
        let len = bytes.len();
        let new_len = len / 2;
        if len <= new_len {
            bytes.clear();
            return;
        }
        bytes.truncate(new_len);
    }

    /// Empties the bytestring. Unless a fake length has been set,
    /// this will emit as the bencode `0:`.
    pub fn clear_content(&'obj mut self) {
        self.bytes.to_mut().clear();
    }

    /// Set the specified [`Range`][std::ops::Range] of bytes in the bytestring to a specified byte.
    pub fn set_content_byterange<R>(&'obj mut self, range: R, byte: u8)
    where
        R: RangeBounds<usize>,
    {
        let len = self.bytes.len();
        let (start, stop) = normalize_range(range, len);
        let bytes = self.bytes.to_mut();
        for i in start..stop {
            let mref = bytes
                .get_mut(i)
                .expect("Tried to replace byte outside of ByteString");
            *mref = byte;
        }
    }
}

impl<'me, 'other, 'ser> InString<'ser>
where
    'other: 'ser,
{
    /// Change the contents of the bytestring. Takes an [`&str`]
    /// and stores a reference to its raw bytes in a [`Cow`].
    pub fn set_content_str(&'me mut self, other: &'other str) {
        self.bytes = Cow::from(other.as_bytes());
    }

    /// Change the contents of the bytestring. Takes a
    /// reference to raw bytes and stores them in a [`Cow`].
    pub fn set_content_u8(&'me mut self, other: &'other [u8]) {
        self.bytes = Cow::from(other);
    }
}

// Not happy about this. Couldn't find a way to get an iterator from a `RangeBounds`.
// And the `Range` type doesn't allow e.g. `..`, `4..`, `..49`, `2..=6` etc to be used.
fn normalize_range<R>(range: R, max_end: usize) -> (usize, usize)
where
    R: RangeBounds<usize>,
{
    // Unstable for now. See: https://github.com/rust-lang/rust/issues/137300
    // if range.is_empty() {
    //     panic!("Can't set byterange contents with an empty range");
    // }

    let beg = range.start_bound();
    let end = range.end_bound();

    let start = match beg {
        core::ops::Bound::Included(n) => *n,
        // Excluded start bound can't be expressed using the range
        // syntax, so I haven't tested it. Probably doesn't matter?
        core::ops::Bound::Excluded(n) => n + 1,
        core::ops::Bound::Unbounded => 0,
    };

    let stop = match end {
        core::ops::Bound::Included(n) => *n + 1,
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

        i.string().set_content_byterange(.., b'0');
        assert_eq!(b"5:00000", i.emit().as_slice());

        let mut i = inspect(b"5:AAAAA");
        i.string().set_content_byterange(0.., b'0');
        assert_eq!(b"5:00000", i.emit().as_slice());

        let mut i = inspect(b"5:AAAAA");
        i.string().set_content_byterange(0..=4, b'0');
        assert_eq!(b"5:00000", i.emit().as_slice());

        let mut i = inspect(b"5:AAAAA");
        i.string().set_content_byterange(0..5, b'0');
        assert_eq!(b"5:00000", i.emit().as_slice());

        let mut i = inspect(b"5:AAAAA");
        i.string().set_content_byterange(0..=0, b'0');
        assert_eq!(b"5:0AAAA", i.emit().as_slice());

        let mut i = inspect(b"5:AAAAA");
        i.string().set_content_byterange(0..1, b'0');
        assert_eq!(b"5:0AAAA", i.emit().as_slice());

        let mut i = inspect(b"5:AAAAA");
        i.string().set_content_byterange(0..=1, b'0');
        assert_eq!(b"5:00AAA", i.emit().as_slice());

        let mut i = inspect(b"5:AAAAA");
        i.string().set_content_byterange(0..2, b'0');
        assert_eq!(b"5:00AAA", i.emit().as_slice());

        let mut i = inspect(b"5:AAAAA");
        i.string().set_content_byterange(0..3, b'0');
        assert_eq!(b"5:000AA", i.emit().as_slice());

        let mut i = inspect(b"5:AAAAA");
        i.string().set_content_byterange(0..4, b'0');
        assert_eq!(b"5:0000A", i.emit().as_slice());

        let mut i = inspect(b"5:AAAAA");
        i.string().set_content_byterange(2..4, b'0');
        assert_eq!(b"5:AA00A", i.emit().as_slice());

        let mut i = inspect(b"5:AAAAA");
        i.string().set_content_byterange(2..5, b'0');
        assert_eq!(b"5:AA000", i.emit().as_slice());
    }

    #[test]
    #[should_panic]
    fn empty_0() {
        let mut i = inspect(b"5:AAAAA");
        i.string().set_content_byterange(0..0, b'0');
    }

    #[test]
    #[should_panic]
    fn empty_1() {
        let mut i = inspect(b"5:AAAAA");
        i.string().set_content_byterange(1..1, b'0');
    }

    #[test]
    #[should_panic]
    fn empty_reversed() {
        let mut i = inspect(b"5:AAAAA");
        i.string().set_content_byterange(5..1, b'0');
    }

    #[test]
    #[should_panic]
    fn out_of_bounds_range_inclusive() {
        let mut i = inspect(b"5:AAAAA");
        i.string().set_content_byterange(2..=5, b'0');
    }

    #[test]
    #[should_panic]
    fn out_of_bounds_range() {
        let mut i = inspect(b"5:AAAAA");
        i.string().set_content_byterange(2..6, b'0');
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
        let l = i.list();
        l.nth(0).replace(Inspectable::new_string(b"aaa"));
        l.nth(1).replace(inspect(b"li1ei2ei3ee"));
        l.nth(2).replace(Inspectable::new_int(5));
        assert_eq!(b"l3:aaali1ei2ei3eei5ee", i.emit().as_slice());
    }
}
