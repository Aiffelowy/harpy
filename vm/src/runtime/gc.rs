use std::collections::HashMap;

use crate::{
    aliases::Result,
    parser::{
        byte_reader::ByteReader,
        function_table::{FunctionIndex, FunctionTable},
        type_table::{Type, TypeTable},
    },
};

use super::{
    heap::Heap,
    operand_stack::OperandStack,
    stack::Stack,
    values::{HeapAddress, StackAddress, VmValue},
};

use crate::parser::header::Header;

use crate::parser::type_table::TypeId;

#[derive(Debug)]
pub struct GarbageCollector {
    bytes_allocated_at_last_gc: usize,
    gc_threshold: usize,
}

impl GarbageCollector {
    pub fn new() -> Self {
        Self {
            bytes_allocated_at_last_gc: 0,
            gc_threshold: 1024,
        }
    }

    pub fn should_collect(&self, current_heap_size: usize) -> bool {
        current_heap_size > self.gc_threshold
    }

    pub fn collect(
        &mut self,
        heap: &mut Heap,
        stack: &mut Stack,
        operand_stack: &mut OperandStack,
        function_table: &FunctionTable,
        type_table: &TypeTable,
        header: &Header,
    ) -> Result<()> {
        let mut address_map = HashMap::new();
        let old_heap_size = heap.bytes_allocated();

        heap.start_copying_gc();

        self.copy_and_update_operand_stack(&mut address_map, operand_stack, heap, type_table);

        self.copy_and_update_stack_frames(
            &mut address_map,
            stack,
            heap,
            function_table,
            type_table,
            header,
        )?;

        heap.finish_copying_gc();

        let new_heap_size = heap.bytes_allocated();
        self.bytes_allocated_at_last_gc = new_heap_size;
        self.gc_threshold = (new_heap_size * 2).max(1024);

        println!(
            "GC: {} -> {} bytes (freed {})",
            old_heap_size,
            new_heap_size,
            old_heap_size - new_heap_size
        );

        Ok(())
    }

    fn copy_and_update_operand_stack(
        &self,
        address_map: &mut HashMap<HeapAddress, HeapAddress>,
        operand_stack: &mut OperandStack,
        heap: &mut Heap,
        type_table: &TypeTable,
    ) {
        for value in operand_stack.iter_mut() {
            if let VmValue::Pointer(old_addr, type_id) = value {
                let new_addr =
                    self.copy_object_if_needed(address_map, *old_addr, *type_id, heap, type_table);
                *old_addr = new_addr; // Update the pointer in-place
            }
        }
    }

    fn copy_and_update_stack_frames(
        &self,
        address_map: &mut HashMap<HeapAddress, HeapAddress>,
        stack: &mut Stack,
        heap: &mut Heap,
        function_table: &FunctionTable,
        type_table: &TypeTable,
        header: &Header,
    ) -> Result<()> {
        let mut current_fp = stack.get_frame_pointer();

        while current_fp.0 >= 16 {
            let func_index = if current_fp.0 == 16 {
                header.main_index
            } else {
                let saved_func_start = current_fp.0 - 16;
                let saved_func_bytes = stack.read_frame_data(saved_func_start + 8, 8);
                let func_idx = usize::from_be_bytes(saved_func_bytes.try_into().unwrap());
                FunctionIndex(func_idx)
            };

            let func_info = &function_table[func_index];

            for (i, &local_type_id) in func_info.local_types.iter().enumerate() {
                if matches!(&type_table[local_type_id], Type::Pointer(_)) {
                    let (local_offset, _) = func_info.local_offsets[i];
                    let local_addr = current_fp.0 + local_offset;

                    let local_type_info = &type_table[local_type_id];
                    let local_data = stack.read_data_at(local_addr, local_type_info.size().0);
                    let mut reader = ByteReader::new(local_data, local_type_info.size().0);

                    if let Ok(VmValue::Pointer(old_addr, pointed_type_id)) =
                        local_type_info.construct(&mut reader)
                    {
                        let new_addr = self.copy_object_if_needed(
                            address_map,
                            old_addr,
                            pointed_type_id,
                            heap,
                            type_table,
                        );

                        let new_pointer = VmValue::Pointer(new_addr, pointed_type_id);
                        stack
                            .write_at(StackAddress(local_addr), new_pointer, local_type_info)
                            .unwrap();
                    }
                }
            }

            if current_fp.0 == 16 {
                break;
            }

            let saved_fp_start = current_fp.0 - 16;
            let saved_fp_bytes = stack.read_frame_data(saved_fp_start, 8);
            let old_fp = usize::from_be_bytes(saved_fp_bytes.try_into().unwrap());
            current_fp = StackAddress(old_fp);
        }

        Ok(())
    }

    fn copy_object_if_needed(
        &self,
        address_map: &mut HashMap<HeapAddress, HeapAddress>,
        old_addr: HeapAddress,
        type_id: TypeId,
        heap: &mut Heap,
        type_table: &TypeTable,
    ) -> HeapAddress {
        if let Some(&new_addr) = address_map.get(&old_addr) {
            new_addr
        } else {
            let size = type_table[type_id].size();
            let new_addr = heap.copy_object_from_old(old_addr, size);
            address_map.insert(old_addr, new_addr);
            new_addr
        }
    }
}
