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
}

impl Heap {
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(1024),
            next_free: 0,
        }
    }

    pub fn alloc(&mut self, ty_size: TypeSize) -> HeapAddress {
        let total_size = 2 + ty_size.0;

        if self.data.len() < self.next_free + total_size {
            self.data.resize(self.next_free + total_size, 0);
        }

        let ref_count_bytes = 1u16.to_be_bytes();
        self.data[self.next_free..self.next_free + 2].copy_from_slice(&ref_count_bytes);

        let data_addr = HeapAddress(self.next_free + 2);
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
}
