use std::io::{self, Write};

/// A value that can be formatted as a decimal integer
pub trait PrintableInteger {
    /// Write the value as a decimal integer
    fn write_to<W: Write>(self, w: W) -> io::Result<()>;
}

macro_rules! impl_integer {
    ($($type:ty)*) => {$(
        impl PrintableInteger for $type {
            fn write_to<W: Write>(self, mut w: W) -> io::Result<()> {
                write!(w, "{}", self)
            }
        }
    )*}
}

impl_integer!(u8 u16 u32 u64 usize i8 i16 i32 i64 isize);
