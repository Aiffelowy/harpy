use crate::{
    aliases::Result,
    err::RuntimeError,
    parser::{
        byte_reader::ByteReader,
        function_table::{FunctionIndex, FunctionTable, LocalIndex},
        type_table::TypeTable,
    },
};

use super::values::{StackAddress, VmValue};

#[derive(Debug)]
pub struct Stack {
    data: Vec<u8>,
    frame_pointer: StackAddress,
    stack_pointer: StackAddress,

    current_function: FunctionIndex,
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
    ) -> Result<VmValue> {
        let (size, local_offset) =
            function_table[self.current_function].local_offsets[local_index.0];
        let offset = self.frame_pointer.0 + local_offset;
        let type_info =
            &type_table[function_table[self.current_function].local_types[local_index.0]];

        let mut reader = ByteReader::new(&self.data[offset..offset + size], size);

        type_info.construct(&mut reader)
    }

    pub fn get_local_address(
        &self,
        function_table: &FunctionTable,
        local_index: LocalIndex,
    ) -> StackAddress {
        let (_, local_offset) = function_table[self.current_function].local_offsets[local_index.0];
        let address = self.frame_pointer.0 + local_offset;
        StackAddress(address)
    }

    pub fn write_local(
        &mut self,
        function_table: &FunctionTable,
        local_index: LocalIndex,
        value: VmValue,
    ) {
        let (size, local_offset) =
            function_table[self.current_function].local_offsets[local_index.0];
        let offset = self.frame_pointer.0 + local_offset;

        value.write_bytes(&mut self.data[offset..offset + size]);
    }

    pub fn set_return_address(&mut self, return_address: usize) {
        self.data[self.frame_pointer.0..self.frame_pointer.0 + 8]
            .copy_from_slice(&return_address.to_be_bytes());
    }

    pub fn get_return_address(&self) -> usize {
        let bytes = &self.data[self.frame_pointer.0..self.frame_pointer.0 + 8];
        usize::from_be_bytes(bytes.try_into().unwrap())
    }

    pub fn push_frame(
        &mut self,
        function_table: &FunctionTable,
        func: FunctionIndex,
    ) -> Result<()> {
        let old_fp = self.frame_pointer;
        let old_fp_bytes = old_fp.0.to_be_bytes();

        let info = &function_table[func];

        let new_frame_start = self.stack_pointer.0;
        let frame_size = info.stack_size;

        if new_frame_start + frame_size + 8 > self.data.len() {
            return Err(RuntimeError::StackOverflow);
        }

        self.data[new_frame_start..new_frame_start + 8].copy_from_slice(&old_fp_bytes);

        self.frame_pointer = StackAddress(new_frame_start + 8);
        self.stack_pointer = StackAddress(self.frame_pointer.0 + frame_size);

        self.data[self.frame_pointer.0..self.stack_pointer.0].fill(0);

        Ok(())
    }

    pub fn is_main(&self) -> bool {
        self.frame_pointer.0 == 8
    }

    pub fn pop_frame(&mut self) -> Result<()> {
        let saved_fp_start = self.frame_pointer.0 - 8;
        let saved_fp_bytes = &self.data[saved_fp_start..saved_fp_start + 8];
        let old_fp = usize::from_be_bytes(saved_fp_bytes.try_into().unwrap());

        self.stack_pointer = StackAddress(saved_fp_start);
        self.frame_pointer = StackAddress(old_fp);

        Ok(())
    }
}
