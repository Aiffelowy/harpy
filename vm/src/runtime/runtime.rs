use crate::{
    aliases::Result,
    err::RuntimeError,
    parser::{
        byte_reader::ByteReader,
        const_pool::{ConstIndex, ConstPool},
        function_table::{CodeAddress, FunctionIndex, FunctionTable, LocalIndex},
        global_table::{GlobalIndex, GlobalTable},
        header::Header,
        type_table::{TypeId, TypeTable},
    },
};

use super::{
    gc::GarbageCollector, heap::Heap, operand_stack::OperandStack, stack::Stack, values::VmValue,
};

static STACK_SIZE: usize = 1048576;

macro_rules! binary_op_runtime {
    ($name:ident) => {
        pub(in crate::runtime) fn $name(&mut self) -> Result<()> {
            let b = self.operand_stack.pop()?;
            let a = self.operand_stack.pop()?;
            self.operand_stack.push(a.$name(b)?);
            Ok(())
        }
    };
}

#[derive(Debug)]
pub struct Runtime<'bytecode> {
    header: Header,
    type_table: TypeTable,
    global_table: GlobalTable,
    const_pool: ConstPool<'bytecode>,
    function_table: FunctionTable,
    pub(in crate::runtime) bytecode: ByteReader<'bytecode>,

    stack: Stack,
    heap: Heap,
    operand_stack: OperandStack,
    gc: GarbageCollector,
}

impl<'bytecode> Runtime<'bytecode> {
    pub fn new(
        header: Header,
        type_table: TypeTable,
        global_table: GlobalTable,
        const_pool: ConstPool<'bytecode>,
        function_table: FunctionTable,
        bytecode: &'bytecode [u8],
    ) -> Result<Self> {
        let bytecode = ByteReader::new(bytecode, bytecode.len());

        Ok(Self {
            stack: Stack::new(header.main_index, &function_table, STACK_SIZE)?,
            type_table,
            global_table,
            const_pool,
            function_table,
            bytecode,
            heap: Heap::new(),
            header,
            operand_stack: OperandStack::new(),
            gc: GarbageCollector::new(),
        })
    }

    pub(in crate::runtime) fn load_const(&mut self, const_id: ConstIndex) {
        self.operand_stack.push(self.const_pool[const_id]);
    }

    pub(in crate::runtime) fn push_addr_local(&mut self, local_id: LocalIndex) {
        let addr = self.stack.get_local_address(&self.function_table, local_id);
        self.operand_stack.push(addr);
    }

    pub(in crate::runtime) fn load_local(&mut self, local_id: LocalIndex) -> Result<()> {
        let value = self.stack.get_local(&self.type_table, &self.function_table, local_id);
        self.operand_stack.push(value);
        Ok(())
    }

    pub(in crate::runtime) fn store_local(&mut self, local_id: LocalIndex) -> Result<()> {
        let v = self.operand_stack.pop()?;
        self.stack.write_local(&self.function_table, local_id, v);
        Ok(())
    }

    pub(in crate::runtime) fn load_global(&mut self, global_id: GlobalIndex) -> Result<()> {
        let v = self.global_table.read_global(global_id, &self.type_table)?;
        self.operand_stack.push(v);
        Ok(())
    }

    pub(in crate::runtime) fn store_global(&mut self, global_id: GlobalIndex) -> Result<()> {
        let v = self.operand_stack.pop()?;
        self.global_table.write_global(global_id, v);
        Ok(())
    }

    pub(in crate::runtime) fn load(&mut self) -> Result<()> {
        let pointer = self.operand_stack.pop()?;

        let value = match pointer {
            VmValue::Pointer(heap_addr, type_id) => {
                self.heap.read_value(heap_addr, type_id, &self.type_table)?
            }
            VmValue::Ref(stack_addr, type_id) => {
                let info = &self.type_table[type_id];
                self.stack.read_at(stack_addr, info)?
            }
            _ => return Err(RuntimeError::InvalidOperation),
        };

        self.operand_stack.push(value);

        Ok(())
    }

    pub(in crate::runtime) fn store(&mut self) -> Result<()> {
        let reference = self.operand_stack.pop()?;
        let value = self.operand_stack.pop()?;

        match reference {
            VmValue::Ref(stack_addr, type_id) => {
                let type_info = &self.type_table[type_id];
                self.stack.write_at(stack_addr, value, type_info)?;
            }
            VmValue::Pointer(heap_addr, type_id) => {
                let type_info = &self.type_table[type_id];
                self.heap.write_value(heap_addr, value, type_info.size());
            }
            _ => return Err(RuntimeError::InvalidOperation),
        }

        Ok(())
    }

    pub(in crate::runtime) fn box_alloc(&mut self, type_id: TypeId) -> Result<()> {
        // Check if we should trigger garbage collection
        if self.gc.should_collect(self.heap.bytes_allocated()) {
            self.gc.collect(
                &mut self.heap,
                &mut self.stack,
                &mut self.operand_stack,
                &mut self.global_table,
                &self.function_table,
                &self.type_table,
                &self.header,
            )?;
        }

        let type_info = &self.type_table[type_id];
        let size = type_info.size();

        let addr = self.heap.alloc(size);

        let value = self.operand_stack.pop()?;
        self.heap.write_value(addr, value, size);

        self.operand_stack.push(VmValue::Pointer(addr, type_id));
        Ok(())
    }

    binary_op_runtime!(add);
    binary_op_runtime!(sub);
    binary_op_runtime!(mul);
    binary_op_runtime!(div);
    binary_op_runtime!(modulo);

    pub(in crate::runtime) fn neg(&mut self) -> Result<()> {
        let v1 = self.operand_stack.pop()?;

        self.operand_stack.push(v1.neg()?);
        Ok(())
    }

    pub(in crate::runtime) fn inc(&mut self) -> Result<()> {
        let v1 = self.operand_stack.pop()?;

        self.operand_stack.push(v1.inc()?);
        Ok(())
    }

    pub(in crate::runtime) fn jmp_condition(
        &mut self,
        address: CodeAddress,
        cond: bool,
    ) -> Result<()> {
        let v = self.operand_stack.pop()?;
        let v = v.as_bool()?;
        if v == cond {
            return self.bytecode.jump_to(address.0 as usize);
        }

        Ok(())
    }

    pub(in crate::runtime) fn call(&mut self, id: FunctionIndex) -> Result<()> {
        let return_address = self.bytecode.position();

        let func_info = &self.function_table[id];

        self.stack.push_frame(&self.function_table, id)?;
        self.stack.set_return_address(return_address);

        for i in (0..func_info.param_count).rev() {
            let arg = self.operand_stack.pop()?;
            self.stack
                .write_local(&self.function_table, LocalIndex(i), arg);
        }

        self.bytecode.jump_to(func_info.code_offset.0 as usize)
    }

    pub(in crate::runtime) fn ret(&mut self) -> Result<()> {
        let return_addr = self.stack.get_return_address();

        self.stack.pop_frame()?;
        self.bytecode.jump_to(return_addr)?;
        Ok(())
    }

    pub(in crate::runtime) fn dup(&mut self) -> Result<()> {
        let v = self.operand_stack.pop()?;
        self.operand_stack.push(v);
        self.operand_stack.push(v);
        Ok(())
    }

    pub(in crate::runtime) fn not(&mut self) -> Result<()> {
        let a = self.operand_stack.pop()?;
        self.operand_stack.push(a.not()?);
        Ok(())
    }

    pub(in crate::runtime) fn halt(&mut self) -> Result<()> {
        if let Ok(v) = self.operand_stack.pop() {
            println!("{}", v.display_with_const_pool(&self.const_pool));
        }
        Err(RuntimeError::Halt)
    }

    pub(in crate::runtime) fn pop(&mut self) -> Result<()> {
        self.operand_stack.pop()?;
        Ok(())
    }

    binary_op_runtime!(and);
    binary_op_runtime!(or);
    binary_op_runtime!(lt);
    binary_op_runtime!(le);
    binary_op_runtime!(gt);
    binary_op_runtime!(ge);
    binary_op_runtime!(eq);
    binary_op_runtime!(ne);

    pub fn run(&'bytecode mut self) -> Result<()> {
        loop {
            let opcode = self.bytecode.read::<u8>()?;
            match crate::runtime::instructions::EXECUTE_TABLE[opcode as usize](self) {
                Ok(()) => (),
                Err(RuntimeError::Halt) => break,
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }
}
