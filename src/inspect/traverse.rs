//! Ways of traversing the AST-like structure of [`Inspectable`]s.
//!
//! Refer to the [documentation here][crate::inspect#traversing].

use smallvec::SmallVec;

use crate::inspect::*;

/// A `Step` in a search path, which selects what next traversal
/// will be applied to an [`Inspectable`] tree.
///
/// A search path is a list of Steps.
#[derive(Debug, Clone)]
pub enum Step<'a> {
    /// Searches for a child, which is either an [`Inspectable`] or a [Dict
    /// Entry][crate::inspect::InDictEntry].
    /// Only finds the first child matching the specific condition.
    Search(Search<'a>),

    /// Access the nth child of a List or Dict.
    Nth(usize),

    /// Accesses a Dict Entry with the given key.
    Entry(&'a [u8]),

    /// Assume current item is an Dict Entry, access the key.
    Key,

    /// Assume current item is an Dict Entry, access the value.
    Value,
}

/// An object that represents what to search for in a [`Step::Search`].
#[derive(Debug, Clone)]
pub enum Search<'a> {
    /// Search for a String with this exact value.
    ByteString(&'a [u8]),

    /// Search for a Dict Entry that has this exact key.
    DictKey(&'a [u8]),

    /// Search for this exact integer.
    Int(Cow<'a, str>),
}

type Path<'a> = SmallVec<[Step<'a>; 20]>;

/// A builder for a path. A path is a list of [`Step`]s undertaken to traverse
/// an [`Inspectable`] AST.
///
/// Use the methods provided to add new steps to the path, then provide a reference
/// to the [`PathBuilder`] to one of these methods:
/// * [`Inspectable::find`]
/// * [`Inspectable::find_ref`]
/// * [`Inspectable::clone_and_mutate`]
/// * [`Inspectable::mutate`]
///
/// Unlike the standard build pattern, no `.build()` function is needed to reify
/// the builder.
///
/// The methods named `into_*` all take ownership of the PathBuilder, add the step,
/// then return the builder. This reduces memory allocation, but may be inconvenient.
///
/// The methods without the `into_` prefix will create a clone of the PathBuilder,
/// add the step to that clone, then return the clone. This may be more convenient
/// if you are building multiple paths that all start with the same steps.
///
/// The builder can contain up to 20 steps before it needs to allocate. For fuzzing,
/// manually building structures using the [`Inspectable`] constructor methods is
/// recommended instead of mutating existing [`Inspectable`]s.
#[derive(Debug, Clone)]
pub struct PathBuilder<'a> {
    pub steps: Path<'a>,
}

impl<'obj, 'pb, 'step> PathBuilder<'pb>
where
    'step: 'pb,
{
    pub fn new() -> Self {
        PathBuilder {
            steps: Default::default(),
        }
    }

    /// Descend into a specific item in a list or entry in a dict, given its index.
    ///
    /// For dicts, MUST be followed by a call to one of: [`.key`][PathBuilder::key],
    /// [`.into_key`][PathBuilder::into_key], [`.value`][PathBuilder::value],
    /// or [`.into_value`][PathBuilder::into_value].
    pub fn into_nth(mut self, idx: usize) -> Self {
        self.steps.push(Step::Nth(idx));
        self
    }

    /// Descend into a specific item in a list or entry in a dict, given its index.
    ///
    /// For dicts, MUST be followed by a call to one of: [`.key`][PathBuilder::key],
    /// [`.into_key`][PathBuilder::into_key], [`.value`][PathBuilder::value],
    /// or [`.into_value`][PathBuilder::into_value].
    pub fn nth(&'obj self, idx: usize) -> Self {
        let next = self.clone();
        next.into_nth(idx)
    }

    /// Access a Dict Entry that has the given key byte string.
    ///
    /// MUST be followed by a call to one of: [`.key`][PathBuilder::key],
    /// [`.into_key`][PathBuilder::into_key], [`.value`][PathBuilder::value],
    /// or [`.into_value`][PathBuilder::into_value].
    pub fn into_entry(mut self, entry_key: &'step [u8]) -> PathBuilder<'pb> {
        self.steps.push(Step::Entry(entry_key));
        self
    }

    /// Access a Dict Entry that has the given key byte string.
    ///
    /// MUST be followed by a call to one of: [`.key`][PathBuilder::key],
    /// [`.into_key`][PathBuilder::into_key], [`.value`][PathBuilder::value],
    /// or [`.into_value`][PathBuilder::into_value].
    pub fn entry(&'obj self, entry_key: &'step [u8]) -> PathBuilder<'pb> {
        let next = self.clone();
        next.into_entry(entry_key)
    }

    /// Access the key part of a Dict Entry.
    pub fn into_key(mut self) -> Self {
        self.steps.push(Step::Key);
        self
    }

    /// Access the key part of a Dict Entry.
    pub fn key(&'obj self) -> Self {
        let next = self.clone();
        next.into_key()
    }

    /// Access the value part of a Dict Entry.
    pub fn into_value(mut self) -> Self {
        self.steps.push(Step::Value);
        self
    }

    /// Access the value part of a Dict Entry.
    pub fn value(&'obj self) -> Self {
        let next = self.clone();
        next.into_value()
    }

    /// Search for byte strings that are not dict keys.
    pub fn into_search_bytestring(mut self, bytestring: &'step [u8]) -> PathBuilder<'pb> {
        self.steps
            .push(Step::Search(Search::ByteString(bytestring)));
        self
    }

    /// Search for byte strings that are not dict keys.
    pub fn search_bytestring(&'obj self, bytestring: &'step [u8]) -> PathBuilder<'pb> {
        let next = self.clone();
        next.into_search_bytestring(bytestring)
    }

    /// Search for Dict Entries that have the given key.
    ///
    /// MUST be followed by a call to one of: [`.key`][PathBuilder::key],
    /// [`.into_key`][PathBuilder::into_key], [`.value`][PathBuilder::value],
    /// or [`.into_value`][PathBuilder::into_value].
    pub fn into_search_entry(mut self, dict_key: &'step [u8]) -> PathBuilder<'pb> {
        self.steps.push(Step::Search(Search::DictKey(dict_key)));
        self
    }

    /// Search for Dict Entries that have the given key.
    ///
    /// MUST be followed by a call to one of: [`.key`][PathBuilder::key],
    /// [`.into_key`][PathBuilder::into_key], [`.value`][PathBuilder::value],
    /// or [`.into_value`][PathBuilder::into_value].
    pub fn search_entry(&'obj self, dict_key: &'step [u8]) -> PathBuilder<'pb> {
        let next = self.clone();
        next.into_search_entry(dict_key)
    }

    /// Search for an integer, provided as a string slice.
    pub fn into_search_int(mut self, number: &'step str) -> PathBuilder<'pb> {
        self.steps.push(Step::Search(Search::Int(number.into())));
        self
    }

    /// Search for an integer, provided as a string slice.
    pub fn search_int(&'obj self, number: &'step str) -> PathBuilder<'pb> {
        let next = self.clone();
        next.into_search_int(number)
    }

    /// Search for an integer, this function converts an i64 to a string slice for you.
    pub fn into_search_int_i64(mut self, number: i64) -> PathBuilder<'pb> {
        self.steps
            .push(Step::Search(Search::Int(number.to_string().into())));
        self
    }

    /// Search for an integer, this function converts an i64 to a string slice for you.
    pub fn search_int_i64(&'obj self, number: i64) -> PathBuilder<'pb> {
        let next = self.clone();
        next.into_search_int_i64(number)
    }
}

impl<'obj, 'ser, 'pb, 'pbobj> Inspectable<'ser> {
    /// Gets a mutable reference to the Inspectable pointed to by the given path.
    /// Panics if no Inspectable matches the given path.
    pub fn find(&'obj mut self, path: &'pbobj PathBuilder<'pb>) -> &'obj mut Inspectable<'ser> {
        let res = self.find_impl(&path.steps, 0);
        match res {
            None => panic!("Path did not resolve to an Inspectable: {:?}", path),
            Some(x) => x,
        }
    }

    /// Gets a reference to the Inspectable pointed to by the given path.
    /// Panics if no Inspectable matches the given path.
    pub fn find_ref(&'obj self, path: &'pbobj PathBuilder<'pb>) -> &'obj Inspectable<'ser> {
        let res = self.find_ref_impl(&path.steps, 0);
        match res {
            None => panic!("Path did not resolve to an Inspectable: {:?}", path),
            Some(x) => x,
        }
    }
}

macro_rules! finders {
    ($(($name:ident, $get:ident, $iter:ident, $( $mutable:ident )? ))*) => {$(
        impl<'obj, 'ser, 'pb, 'pbobj> Inspectable<'ser> {
            fn $name(
                &'obj $($mutable)? self,
                steps: &'pbobj Path<'pb>,
                pc: usize,
            ) -> Option<&'obj $($mutable)? Inspectable<'ser>> {
                let current_step = if let Some(x) = steps.get(pc) {
                    x
                } else {
                    return Some(self);
                };

                let descend_into_entry =
                    |entry: &'obj $($mutable)? InDictEntry<'ser>| -> Option<&'obj $($mutable)? Inspectable<'ser>> {
                        match steps.get(pc + 1) {
                            Some(Step::Key) => entry.key.$name(steps, pc + 2),
                            Some(Step::Value) => entry.value.$name(steps, pc + 2),
                            _ => panic!(
                                "A path that selects a dict entry must then select either its key or its value"
                            ),
                        }
                    };

                match (self, current_step) {
                    (Inspectable::Raw(_), _) => (),

                    (s @ Inspectable::Int(_), Step::Search(Search::Int(x))) => {
                        if *x == &*s.int_ref().bytes {
                            return Some(s);
                        }
                    },
                    (Inspectable::Int(_), _) => (),

                    (s @ Inspectable::String(_), Step::Search(Search::ByteString(x))) => {
                        if *x == &*s.string_ref().bytes {
                            return Some(s);
                        }
                    },
                    (Inspectable::String(_), Step::Search(_)) => (),
                    (Inspectable::String(_), _) => (),

                    (Inspectable::List(_), Step::Key) => (),
                    (Inspectable::List(_), Step::Value) => (),
                    (Inspectable::List(_), Step::Entry(_)) => (),
                    (Inspectable::List(in_list), Step::Nth(idx)) => {
                        let item = in_list.items.$get(*idx)?;
                        return item.$name(steps, pc + 1);
                    },
                    (Inspectable::List(in_list), Step::Search(_)) => {
                        return in_list
                            .items
                            .$iter()
                            .find_map(|item| item.$name(steps, pc));
                        },

                        (Inspectable::Dict(_), Step::Key) => (),
                        (Inspectable::Dict(_), Step::Value) => (),
                        (Inspectable::Dict(in_dict), Step::Nth(idx)) => {
                            let entry = in_dict.items.$get(*idx)?;
                            return descend_into_entry(entry);
                        },
                        (Inspectable::Dict(in_dict), Step::Entry(key)) => {
                            let entry = in_dict.items.$iter().find(|entry| {
                                let entry_key = if let Inspectable::String(x) = &entry.key {
                                    x
                                } else {
                                    return false;
                                };
                                entry_key.bytes == *key
                            })?;
                            return descend_into_entry(entry);
                        },
                        (Inspectable::Dict(in_dict), Step::Search(Search::DictKey(key))) => {
                            return in_dict.items.$iter().find_map(|entry| {
                                if let Inspectable::String(x) = &entry.key {
                                    if x.bytes == *key {
                                        return descend_into_entry(entry);
                                    }
                                }
                                entry.value.$name(steps, pc)
                            });
                        },
                        (Inspectable::Dict(in_dict), Step::Search(_)) => {
                            return in_dict
                                .items
                                .$iter()
                                .find_map(|InDictEntry { value, .. }| value.$name(steps, pc));
                            },
                };
                None
            }
        }
    )*};
}

finders!(
    (find_impl, get_mut, iter_mut, mut)
    (find_ref_impl, get, iter,)
);

macro_rules! variant_accessors {
    ($(($name:ident, $mutname:ident, $source:ident, $target:ty, $targettype:ty))*) => {$(
        impl<'obj, 'ser> Inspectable<'ser> {
            #[doc=concat!("Coercion to immutable reference to [`", stringify!($targettype), "`]. Panics on failure.")]
            pub fn $name(&'obj self) -> &'obj $target {
                match self {
                    Inspectable::$source(x) => x,
                    _ => panic!("Attempted to take non-{} Inspectable as {}", stringify!($source), stringify!($target))
                }
            }
            #[doc=concat!("Coercion to mutable reference to [`", stringify!($targettype), "`]. Panics on failure.")]
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
    (string_ref, string, String, InString<'ser>, InString)
    (int_ref, int, Int, InInt<'ser>, InInt)
    (raw_ref, raw, Raw, Cow<'ser, [u8]>, [u8])
    (dict_ref, dict, Dict, InDict<'ser>, InDict)
    (list_ref, list, List, InList<'ser>, InList)
}

impl<'obj, 'ser> InList<'ser> {
    pub fn nth_ref(&'obj self, idx: usize) -> &'obj Inspectable<'ser> {
        self.items
            .get(idx)
            .expect("Could not access nth Inspectable in list")
    }

    pub fn nth(&'obj mut self, idx: usize) -> &'obj mut Inspectable<'ser> {
        self.items
            .get_mut(idx)
            .expect("Could not mutably access nth Inspectable in list")
    }
}

impl<'obj, 'ser> InDict<'ser> {
    pub fn nth_ref(&'obj self, idx: usize) -> &'obj InDictEntry<'ser> {
        self.items
            .get(idx)
            .expect("Could not access nth Inspectable in list")
    }

    pub fn nth(&'obj mut self, idx: usize) -> &'obj mut InDictEntry<'ser> {
        self.items
            .get_mut(idx)
            .expect("Could not mutably access nth Inspectable in list")
    }
}

impl<'obj, 'ser, 'other> InDict<'ser> {
    pub fn entry_ref(&'obj self, name: &'other [u8]) -> &'obj InDictEntry<'ser> {
        self.items
            .iter()
            .find(|InDictEntry { key, .. }| key.string_ref().bytes == name)
            .expect("Could not find a Dict Entry with requested key")
    }

    pub fn entry(&'obj mut self, name: &'other [u8]) -> &'obj mut InDictEntry<'ser> {
        self.items
            .iter_mut()
            .find(|InDictEntry { key, .. }| key.string_ref().bytes == name)
            .expect("Could not find a Dict Entry with requested key")
    }
}
