use crate::{aliases::Result, err::RuntimeError};

use super::values::VmValue;

#[derive(Debug)]
pub struct OperandStack {
    data: Vec<VmValue>,
}

impl OperandStack {
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(128),
        }
    }

    pub fn push(&mut self, value: VmValue) {
        self.data.push(value);
    }

    pub fn pop(&mut self) -> Result<VmValue> {
        self.data.pop().ok_or(RuntimeError::BadStack)
    }
}
