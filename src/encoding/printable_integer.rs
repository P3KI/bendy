#[cfg(not(feature = "std"))]
use core::fmt::Display;
#[cfg(feature = "std")]
use std::fmt::Display;

/// A value that can be formatted as a decimal integer
pub trait PrintableInteger: Display {}

macro_rules! impl_integer {
    ($($type:ty)*) => {$(
        impl PrintableInteger for $type {}
    )*}
}

impl_integer!(u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize);
