use crate::{
    aliases::Result,
    parser::{
        byte_reader::ByteReader,
        const_pool::{ConstIndex, ConstPool},
        function_table::{CodeAddress, FunctionIndex, FunctionTable, LocalIndex},
        header::Header,
        type_table::TypeTable,
    },
};

use super::{
    heap::Heap, instructions::Instruction, operand_stack::OperandStack, stack::Stack,
    values::VmValue,
};

static STACK_SIZE: usize = 1048576;

macro_rules! binary_op_runtime {
    ($name:ident, $method:ident) => {
        fn $name(&mut self) -> Result<()> {
            let b = self.operand_stack.pop()?;
            let a = self.operand_stack.pop()?;
            self.operand_stack.push(a.$method(b)?);
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
            heap: Heap {},
            header,
            operand_stack: OperandStack::new(),
            running: true,
        })
    }

    fn load_const(&mut self, const_id: ConstIndex) {
        self.operand_stack.push(self.const_pool[const_id]);
    }

    fn push_addr_local(&mut self, local_id: LocalIndex) {
        let addr = self.stack.get_local_address(&self.function_table, local_id);
        self.operand_stack.push(VmValue::Ref(addr));
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

    fn load(&mut self) {
        todo!()
    }

    fn store(&mut self) {
        todo!()
    }

    fn box_alloc(&mut self) {
        todo!()
    }

    fn add(&mut self) -> Result<()> {
        let v1 = self.operand_stack.pop()?;
        let v2 = self.operand_stack.pop()?;

        self.operand_stack.push(v2.add(v1)?);
        Ok(())
    }

    fn sub(&mut self) -> Result<()> {
        let v1 = self.operand_stack.pop()?;
        let v2 = self.operand_stack.pop()?;

        self.operand_stack.push(v2.sub(v1)?);
        Ok(())
    }

    fn mul(&mut self) -> Result<()> {
        let v1 = self.operand_stack.pop()?;
        let v2 = self.operand_stack.pop()?;

        self.operand_stack.push(v2.mul(v1)?);
        Ok(())
    }

    fn div(&mut self) -> Result<()> {
        let v1 = self.operand_stack.pop()?;
        let v2 = self.operand_stack.pop()?;

        self.operand_stack.push(v2.div(v1)?);
        Ok(())
    }

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
        let v = self.operand_stack.pop()?.as_bool()?;
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

    fn and(&mut self) -> Result<()> {
        let b = self.operand_stack.pop()?;
        let a = self.operand_stack.pop()?;
        self.operand_stack.push(a.and(b)?);
        Ok(())
    }

    fn or(&mut self) -> Result<()> {
        let b = self.operand_stack.pop()?;
        let a = self.operand_stack.pop()?;
        self.operand_stack.push(a.or(b)?);
        Ok(())
    }

    fn not(&mut self) -> Result<()> {
        let a = self.operand_stack.pop()?;
        self.operand_stack.push(a.not()?);
        Ok(())
    }

    binary_op_runtime!(lt, lt);
    binary_op_runtime!(le, le);
    binary_op_runtime!(gt, gt);
    binary_op_runtime!(ge, ge);
    binary_op_runtime!(eq, eq);
    binary_op_runtime!(ne, ne);

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

                LOAD => self.load(),
                STORE => self.store(),

                BOX_ALLOC => self.box_alloc(),

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
