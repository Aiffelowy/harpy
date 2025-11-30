use crate::{
    aliases::Result,
    err::RuntimeError,
    parser::{
        byte_reader::ByteReader,
        function_table::{FunctionIndex, FunctionTable, LocalIndex},
        type_table::{Type, TypeTable},
    },
};

use super::values::{StackAddress, VmValue};

#[derive(Debug)]
pub struct Stack {
    pub data: Vec<u8>,
    pub frame_pointer: StackAddress,
    pub stack_pointer: StackAddress,

    pub current_function: FunctionIndex,
}

impl Stack {
    pub fn new(
        main_id: FunctionIndex,
        function_table: &FunctionTable,
        size: usize,
    ) -> Result<Self> {
        let mut s = Self {
            data: vec![0u8; size],
            frame_pointer: StackAddress(0),
            stack_pointer: StackAddress(0),
            current_function: FunctionIndex(0),
        };

        s.push_frame(function_table, main_id)?;
        Ok(s)
    }

    pub fn get_local(
        &self,
        type_table: &TypeTable,
        function_table: &FunctionTable,
        local_index: LocalIndex,
    ) -> VmValue {
        let func_info = &function_table[self.current_function];
        let (local_offset, size) = unsafe { *func_info.local_offsets.get_unchecked(local_index.0) };
        let type_id = unsafe { *func_info.local_types.get_unchecked(local_index.0) };
        let type_info = &type_table[type_id];

        let offset = self.frame_pointer.0 + local_offset;
        let data = unsafe { self.data.get_unchecked(offset..offset + size) };

        let mut reader = ByteReader::new(data, size);
        unsafe { type_info.construct(&mut reader).unwrap_unchecked() }
    }

    pub fn get_local_address(
        &self,
        function_table: &FunctionTable,
        local_index: LocalIndex,
    ) -> VmValue {
        let (local_offset, _) = function_table[self.current_function].local_offsets[local_index.0];
        let id = function_table[self.current_function].local_types[local_index.0];
        let address = self.frame_pointer.0 + local_offset;
        VmValue::Ref(StackAddress(address), id)
    }

    pub fn write_local(
        &mut self,
        function_table: &FunctionTable,
        local_index: LocalIndex,
        value: VmValue,
    ) {
        let (local_offset, size) =
            function_table[self.current_function].local_offsets[local_index.0];
        let offset = self.frame_pointer.0 + local_offset;

        unsafe { value.write_bytes(self.data.get_unchecked_mut(offset..offset + size)) };
    }

    pub fn set_return_address(&mut self, return_address: usize) {
        self.data[self.frame_pointer.0..self.frame_pointer.0 + 8]
            .copy_from_slice(&return_address.to_be_bytes());
    }

    pub fn get_return_address(&self) -> usize {
        let bytes = unsafe {
            self.data
                .get_unchecked(self.frame_pointer.0..self.frame_pointer.0 + 8)
        };
        usize::from_be_bytes(bytes.try_into().unwrap())
    }

    pub fn push_frame(
        &mut self,
        function_table: &FunctionTable,
        func: FunctionIndex,
    ) -> Result<()> {
        let old_fp = self.frame_pointer;
        let old_fp_bytes = old_fp.0.to_be_bytes();
        let old_func_bytes = self.current_function.0.to_be_bytes();

        let info = &function_table[func];

        let new_frame_start = self.stack_pointer.0;
        let frame_size = info.stack_size;

        if new_frame_start + frame_size + 16 > self.data.len() {
            return Err(RuntimeError::StackOverflow);
        }

        self.data[new_frame_start..new_frame_start + 8].copy_from_slice(&old_fp_bytes);
        self.data[new_frame_start + 8..new_frame_start + 16].copy_from_slice(&old_func_bytes);

        self.frame_pointer = StackAddress(new_frame_start + 16);
        self.stack_pointer = StackAddress(self.frame_pointer.0 + frame_size);
        self.current_function = func;

        //self.data[self.frame_pointer.0..self.stack_pointer.0].fill(0);

        Ok(())
    }

    pub fn pop_frame(&mut self) -> Result<()> {
        let saved_fp_start = self.frame_pointer.0 - 16;
        let saved_fp_bytes = &self.data[saved_fp_start..saved_fp_start + 8];
        let saved_func_bytes = &self.data[saved_fp_start + 8..saved_fp_start + 16];
        let old_fp = usize::from_be_bytes(saved_fp_bytes.try_into().unwrap());
        let old_func = usize::from_be_bytes(saved_func_bytes.try_into().unwrap());

        self.stack_pointer = StackAddress(saved_fp_start);
        self.frame_pointer = StackAddress(old_fp);
        self.current_function = FunctionIndex(old_func);

        Ok(())
    }

    pub fn read_at(&self, addr: StackAddress, type_info: &Type) -> Result<VmValue> {
        let mut reader = ByteReader::new(&self.data[addr.0..], type_info.size().0);
        type_info.construct(&mut reader)
    }

    pub fn write_at(&mut self, addr: StackAddress, value: VmValue, type_info: &Type) -> Result<()> {
        let size = type_info.size().0;
        let memory_slice = &mut self.data[addr.0..addr.0 + size];
        value.write_bytes(memory_slice);
        Ok(())
    }

    // GC support methods
    pub fn get_frame_pointer(&self) -> StackAddress {
        self.frame_pointer
    }

    pub fn read_frame_data(&self, addr: usize, len: usize) -> &[u8] {
        &self.data[addr..addr + len]
    }

    pub fn read_data_at(&self, addr: usize, len: usize) -> &[u8] {
        &self.data[addr..addr + len]
    }
}
