use crate::{
    aliases::Result,
    parser::{
        byte_reader::ByteReader,
        type_table::{TypeId, TypeSize, TypeTable},
    },
};

use super::values::{HeapAddress, VmValue};

#[derive(Debug)]
pub struct Heap {
    data: Vec<u8>,
    next_free: usize,

    copy_buffer: Vec<u8>,
    copy_next_free: usize,
}

impl Heap {
    pub fn new() -> Self {
        let initial_capacity = 1024 * 1024;
        Self {
            data: Vec::with_capacity(initial_capacity),
            next_free: 0,
            copy_buffer: Vec::with_capacity(initial_capacity),
            copy_next_free: 0,
        }
    }

    pub fn alloc(&mut self, ty_size: TypeSize) -> HeapAddress {
        let total_size = ty_size.0;

        if self.data.len() < self.next_free + total_size {
            self.data.resize(self.next_free + total_size, 0);
        }

        let data_addr = HeapAddress(self.next_free);
        self.next_free += total_size;

        data_addr
    }

    pub fn write_value(&mut self, addr: HeapAddress, value: VmValue, size: TypeSize) {
        let memory_slice = &mut self.data[addr.0..addr.0 + size.0];
        value.write_bytes(memory_slice);
    }

    pub fn read_value(
        &mut self,
        addr: HeapAddress,
        type_id: TypeId,
        type_table: &TypeTable,
    ) -> Result<VmValue> {
        let type_info = &type_table[type_id];
        let mut reader = ByteReader::new(&self.data[addr.0..], type_info.size().0);
        type_info.construct(&mut reader)
    }

    // GC support methods
    pub fn bytes_allocated(&self) -> usize {
        self.next_free
    }

    pub fn start_copying_gc(&mut self) {
        self.copy_buffer.clear();
        self.copy_buffer.resize(self.data.len(), 0);
        self.copy_next_free = 0;
    }

    pub fn copy_object_from_old(&mut self, old_addr: HeapAddress, size: TypeSize) -> HeapAddress {
        let old_data = &self.data[old_addr.0..old_addr.0 + size.0];

        let new_addr = HeapAddress(self.copy_next_free);
        self.copy_buffer[self.copy_next_free..self.copy_next_free + size.0]
            .copy_from_slice(old_data);
        self.copy_next_free += size.0;

        new_addr
    }

    pub fn finish_copying_gc(&mut self) {
        std::mem::swap(&mut self.data, &mut self.copy_buffer);
        self.next_free = self.copy_next_free;
        self.copy_next_free = 0;
    }
}
