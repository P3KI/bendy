#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

pub trait Stack<T> {
    fn peek_mut(&mut self) -> Option<&mut T>;

    fn peek(&self) -> Option<&T>;

    fn replace_top(&mut self, new_value: T);
}

impl<T> Stack<T> for Vec<T> {
    fn peek_mut(&mut self) -> Option<&mut T> {
        let len = self.len();
        if len == 0 {
            None
        } else {
            Some(&mut self[len - 1])
        }
    }

    fn peek(&self) -> Option<&T> {
        let len = self.len();
        if len == 0 {
            None
        } else {
            Some(&self[len - 1])
        }
    }

    fn replace_top(&mut self, new_value: T) {
        self.peek_mut()
            .map(|top| *top = new_value)
            .expect("Shouldn't replace_top with nothing on the stack");
    }
}
