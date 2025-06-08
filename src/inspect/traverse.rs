use crate::inspect::*;

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


impl<'obj, 'ser> InList<'ser> {
    pub fn nth(&'obj self, idx: usize) -> &'obj Inspectable<'ser> {
        self.items.get(idx).expect("Could not access nth Inspectable in list")
    }

    pub fn nth_mut(&'obj mut self, idx: usize) -> &'obj mut Inspectable<'ser> {
        self.items.get_mut(idx).expect("Could not mutably access nth Inspectable in list")
    }
}

impl<'obj, 'ser> InDict<'ser> {
    pub fn nth(&'obj self, idx: usize) -> &'obj InTuple<'ser> {
        self.items.get(idx).expect("Could not access nth Inspectable in list")
    }

    pub fn nth_mut(&'obj mut self, idx: usize) -> &'obj mut InTuple<'ser> {
        self.items.get_mut(idx).expect("Could not mutably access nth Inspectable in list")
    }
}

impl<'obj, 'ser, 'other> InDict<'ser> {
    pub fn entry(&'obj self, name: &'other [u8]) -> &'obj InTuple<'ser> {
        self.items.iter().find(|InTuple{key, ..}| key.string().bytes == name)
            .expect("Could not find a tuple with requested key")
    }

    pub fn entry_mut(&'obj mut self, name: &'other [u8]) -> &'obj mut InTuple<'ser> {
        self.items.iter_mut().find(|InTuple{key, ..}| key.string().bytes == name)
            .expect("Could not find a tuple with requested key")
    }
}
