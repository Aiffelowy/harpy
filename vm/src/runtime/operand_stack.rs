use crate::{aliases::Result, err::RuntimeError};

use super::values::VmValue;

#[derive(Debug)]
pub struct OperandStack {
    data: Vec<VmValue>,
}

impl OperandStack {
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(512),
        }
    }

    pub fn push(&mut self, value: VmValue) {
        self.data.push(value);
    }

    pub fn pop(&mut self) -> Result<VmValue> {
        self.data.pop().ok_or(RuntimeError::BadStack)
    }

    // GC support methods
    pub fn iter(&self) -> impl Iterator<Item = &VmValue> {
        self.data.iter()
    }
    
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut VmValue> {
        self.data.iter_mut()
    }
}
