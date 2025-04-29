#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

pub trait Stack<T> {
    fn peek(&self) -> Option<&T>;
}

impl<T> Stack<T> for Vec<T> {
    fn peek(&self) -> Option<&T> {
        let len = self.len();
        if len == 0 {
            None
        } else {
            Some(&self[len - 1])
        }
    }
}
