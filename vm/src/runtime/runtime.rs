use crate::{
    aliases::Result,
    err::RuntimeError,
    parser::{
        byte_reader::ByteReader,
        const_pool::{ConstIndex, ConstPool},
        function_table::{CodeAddress, FunctionIndex, FunctionTable, LocalIndex},
        header::Header,
        type_table::{TypeId, TypeTable},
    },
};

use super::{
    gc::GarbageCollector, heap::Heap, instructions::Instruction, operand_stack::OperandStack, stack::Stack,
    values::VmValue,
};

static STACK_SIZE: usize = 1048576;

macro_rules! binary_op_runtime {
    ($name:ident) => {
        fn $name(&mut self) -> Result<()> {
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
    const_pool: ConstPool<'bytecode>,
    function_table: FunctionTable,
    bytecode: ByteReader<'bytecode>,

    stack: Stack,
    heap: Heap,
    operand_stack: OperandStack,
    gc: GarbageCollector,
    running: bool,
}

impl<'bytecode> Runtime<'bytecode> {
    pub fn new(
        header: Header,
        type_table: TypeTable,
        const_pool: ConstPool<'bytecode>,
        function_table: FunctionTable,
        bytecode: &'bytecode [u8],
    ) -> Result<Self> {
        let mut bytecode = ByteReader::new(bytecode, bytecode.len());
        bytecode.jump_to(function_table[header.entry_point].code_offset.0 as usize)?;

        Ok(Self {
            stack: Stack::new(header.entry_point, &function_table, STACK_SIZE)?,
            type_table,
            const_pool,
            function_table,
            bytecode,
            heap: Heap::new(),
            header,
            operand_stack: OperandStack::new(),
            gc: GarbageCollector::new(),
            running: true,
        })
    }

    fn load_const(&mut self, const_id: ConstIndex) {
        self.operand_stack.push(self.const_pool[const_id]);
    }

    fn push_addr_local(&mut self, local_id: LocalIndex) {
        let addr = self.stack.get_local_address(&self.function_table, local_id);
        self.operand_stack.push(addr);
    }

    fn load_local(&mut self, local_id: LocalIndex) -> Result<()> {
        let v = self
            .stack
            .get_local(&self.type_table, &self.function_table, local_id)?;
        self.operand_stack.push(v);
        Ok(())
    }

    fn store_local(&mut self, local_id: LocalIndex) -> Result<()> {
        let v = self.operand_stack.pop()?;
        self.stack.write_local(&self.function_table, local_id, v);
        Ok(())
    }

    fn load(&mut self) -> Result<()> {
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

    fn store(&mut self) -> Result<()> {
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

    fn box_alloc(&mut self, type_id: TypeId) -> Result<()> {
        // Check if we should trigger garbage collection
        if self.gc.should_collect(self.heap.bytes_allocated()) {
            self.gc.collect(
                &mut self.heap,
                &mut self.stack,
                &mut self.operand_stack,
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

    fn neg(&mut self) -> Result<()> {
        let v1 = self.operand_stack.pop()?;

        self.operand_stack.push(v1.neg()?);
        Ok(())
    }

    fn inc(&mut self) -> Result<()> {
        let v1 = self.operand_stack.pop()?;

        self.operand_stack.push(v1.inc()?);
        Ok(())
    }

    fn jmp_condition(&mut self, address: CodeAddress, cond: bool) -> Result<()> {
        let v = self.operand_stack.pop()?;
        let v = v.as_bool()?;
        if v == cond {
            return self.bytecode.jump_to(address.0 as usize);
        }

        Ok(())
    }

    fn call(&mut self, id: FunctionIndex) -> Result<()> {
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

    fn ret(&mut self) -> Result<()> {
        let return_addr = self.stack.get_return_address();

        if self.stack.is_main() {
            self.running = false;
            if let Ok(v) = self.operand_stack.pop() {
                println!("{:?}", v);
            }
            return Ok(());
        }

        self.stack.pop_frame()?;
        self.bytecode.jump_to(return_addr)?;
        Ok(())
    }

    fn dup(&mut self) -> Result<()> {
        let v = self.operand_stack.pop()?;
        self.operand_stack.push(v);
        self.operand_stack.push(v);
        Ok(())
    }

    fn not(&mut self) -> Result<()> {
        let a = self.operand_stack.pop()?;
        self.operand_stack.push(a.not()?);
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

    pub fn run(&mut self) -> Result<()> {
        use Instruction::*;
        while self.running {
            let instruction = self.bytecode.read_safe()?;
            match instruction {
                NOP => (),

                LOAD_CONST(id) => self.load_const(id),

                PUSH_ADDR_LOCAL(li) => self.push_addr_local(li),
                LOAD_LOCAL(li) => self.load_local(li)?,
                STORE_LOCAL(li) => self.store_local(li)?,

                LOAD => self.load()?,
                STORE => self.store()?,

                BOX_ALLOC(id) => self.box_alloc(id)?,

                ADD => self.add()?,
                SUB => self.sub()?,
                MUL => self.mul()?,
                DIV => self.div()?,
                NEG => self.neg()?,
                INC => self.inc()?,

                JMP(ca) => self.bytecode.jump_to(ca.0 as usize)?,
                JMP_IF_TRUE(ca) => self.jmp_condition(ca, true)?,
                JMP_IF_FALSE(ca) => self.jmp_condition(ca, false)?,

                CALL(fi) => self.call(fi)?,
                RET => self.ret()?,

                EQ => self.eq()?,
                NEQ => self.ne()?,
                LT => self.lt()?,
                LTE => self.le()?,
                GT => self.gt()?,
                GTE => self.ge()?,

                AND => self.and()?,
                OR => self.or()?,
                NOT => self.not()?,

                POP => {
                    self.operand_stack.pop()?;
                }
                DUP => self.dup()?,

                HALT => break,
            }
        }

        Ok(())
    }
}
