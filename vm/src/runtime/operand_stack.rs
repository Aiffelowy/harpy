use crate::{aliases::Result, err::RuntimeError};

use super::values::VmValue;

const MAX_STACK_SIZE: usize = 32;

#[derive(Debug)]
pub struct OperandStack {
    data: [VmValue; MAX_STACK_SIZE],
    len: usize,
}

impl OperandStack {
    pub fn new() -> Self {
        Self {
            data: [VmValue::Int(0); MAX_STACK_SIZE],
            len: 0,
        }
    }

    #[inline(always)]
    pub fn push(&mut self, value: VmValue) {
        if self.len >= MAX_STACK_SIZE {
            panic!("Operand stack overflow");
        }
        unsafe {
            *self.data.get_unchecked_mut(self.len) = value;
        }
        self.len += 1;
    }

    #[inline(always)]
    pub fn pop(&mut self) -> Result<VmValue> {
        if self.len == 0 {
            return Err(RuntimeError::BadStack);
        }
        self.len -= 1;
        unsafe { Ok(*self.data.get_unchecked(self.len)) }
    }

    // GC support methods
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut VmValue> {
        self.data[..self.len].iter_mut()
    }
}
