use std::collections::HashMap;

use crate::{
    aliases::{MAGIC_NUMBER, VERSION},lexer::tokens::Lit, parser::{expr::Expr, node::NodeId, program::Program, types::runtime::RuntimeType}, semantic_analyzer::{
        const_pool::ConstIndex, function_table::FuncIndex, result::RuntimeAnalysisResult,
    }
};

use super::{
    compile_trait::Generate,
    expr_generators::expr_gen::ExprGenerator,
    instruction::{Instruction, Label, LocalAddress},
};

static PREALLOC_CODE_BUFFER: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BytecodeNode {
    Instruction(Instruction),
    Label(Label),
}

#[derive(Debug)]
pub struct Generator {
    code: Vec<BytecodeNode>,
    function_entry_points: HashMap<FuncIndex, Label>,
    analysis_result: RuntimeAnalysisResult,

    main_index: FuncIndex,
    next_label: u64,
}

impl Generator {
    fn new(analysis: RuntimeAnalysisResult) -> Self {
        Self {
            code: Vec::with_capacity(PREALLOC_CODE_BUFFER),
            function_entry_points: HashMap::new(),
            analysis_result: analysis,
            main_index: FuncIndex(0),
            next_label: 0,
        }
    }

    pub fn gen<G: Generate>(&mut self, thing: &G) {
        thing.generate(self);
    }

    pub fn gen_expr(&mut self, expr: &Expr) {
        ExprGenerator::generate(expr, self);
    }

    pub fn create_label(&mut self) -> Label {
        let label = Label(self.next_label);
        self.next_label += 1;
        label
    }

    pub fn set_main(&mut self, idx: FuncIndex) {
        self.main_index = idx;
    }

    pub fn register_function(&mut self, function: FuncIndex, label: Label) {
        self.function_entry_points.insert(function, label);
    }

    pub fn place_label(&mut self, label: Label) {
        self.code.push(BytecodeNode::Label(label));
    }

    pub fn push_instruction(&mut self, instruction: Instruction) {
        self.code.push(BytecodeNode::Instruction(instruction));
    }

    pub fn get_local_mapping(&self, id: NodeId) -> LocalAddress {
        self.analysis_result.locals_map[&id]
    }

    pub fn get_call_mapping(&self, id: NodeId) -> FuncIndex {
        self.analysis_result.function_table.get_mapping(id)
    }

    pub fn get_const_mapping(&self, id: NodeId) -> ConstIndex {
        self.analysis_result.constants.get_mapping(id)
    }

    pub fn place_ret(&mut self) {
        if BytecodeNode::Instruction(Instruction::RET) == self.code[self.code.len() - 1] {
            return;
        }

        self.code.push(BytecodeNode::Instruction(Instruction::RET));
    }

    pub fn get_function_mapping(&self, id: NodeId) -> FuncIndex {
        self.analysis_result
            .function_table
            .get_function_delc_mapping(id)
    }

    fn generate_type_table(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();

        for ty in self.analysis_result.type_table.iter() {
            match &ty.ttype {
                RuntimeType::Base(b) => {
                    match &b {
                        crate::parser::types::BaseType::Primitive(p) => {
                            data.push(0x01);
                            data.push(p.type_id());
                        }
                        crate::parser::types::BaseType::Custom(_) => {
                            data.push(0x04);
                        }
                    }
                    data.push(ty.size);
                }
                RuntimeType::Boxed(i) => {
                    data.push(0x02);
                    data.extend(i.0.to_be_bytes());
                }
                RuntimeType::Ref(i) => {
                    data.push(0x03);
                    data.extend(i.0.to_be_bytes());
                }
                RuntimeType::Void => {
                    data.push(0x00);
                }
            }
        }

        data
    }

    fn generate_const_pool(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();

        for con in self.analysis_result.constants.iter() {
            data.extend(con.type_idx.0.to_be_bytes());
            match &con.lit {
                Lit::LitVoid => (),
                Lit::LitInt(i) => data.extend(i.to_be_bytes()),
                Lit::LitBool(b) => data.push(*b as u8),
                Lit::LitFloat(f) => data.extend(f.to_be_bytes()),
                Lit::LitStr(s) => {
                    data.extend((s.len() as u64).to_be_bytes());
                    data.extend(s.as_bytes());
                }
            }
        }

        data
    }

    fn generate_function_table(&self, label_positions: &HashMap<Label, u64>) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();

        for (i, func) in self.analysis_result.function_table.iter().enumerate() {
            let label = self.function_entry_points[&FuncIndex(i as u32)];
            let code_offset = label_positions.get(&label).unwrap_or(&0);
            data.extend(code_offset.to_be_bytes());
            data.extend((func.params.len() as u16).to_be_bytes());
            data.extend((func.locals.len() as u16).to_be_bytes());
            for ty in &func.locals {
                data.extend(ty.0.to_be_bytes())
            }
        }

        data
    }

    fn generate_bytecode(&self) -> (Vec<u8>, HashMap<Label, u64>) {
        let mut data = Vec::new();
        let mut label_positions: HashMap<Label, u64> = HashMap::new();
        let mut position = 0u64;

        for node in &self.code {
            match node {
                BytecodeNode::Label(label) => {
                    label_positions.insert(*label, position);
                }
                BytecodeNode::Instruction(instr) => {
                    position += self.instruction_size(&instr);
                }
            }
        }

        for node in &self.code {
            if let BytecodeNode::Instruction(instr) = node {
                data.push(instr.opcode());

                match instr {
                    Instruction::LOAD_CONST(idx) => {
                        data.extend(idx.0.to_be_bytes());
                    }
                    Instruction::PUSH_ADDR_LOCAL(addr)
                    | Instruction::LOAD_LOCAL(addr)
                    | Instruction::STORE_LOCAL(addr) => {
                        data.extend(addr.0.to_be_bytes());
                    }
                    Instruction::JMP(label)
                    | Instruction::JMP_IF_TRUE(label)
                    | Instruction::JMP_IF_FALSE(label) => {
                        let target_pos = label_positions.get(label).unwrap_or(&0);
                        data.extend(target_pos.to_be_bytes());
                    }
                    Instruction::CALL(func_idx) => {
                        data.extend(func_idx.0.to_be_bytes());
                    }
                    _ => {}
                }
            }
        }

        (data, label_positions)
    }

    fn instruction_size(&self, instr: &Instruction) -> u64 {
        match instr {
            Instruction::LOAD_CONST(_) => 1 + 4,
            Instruction::PUSH_ADDR_LOCAL(_)
            | Instruction::LOAD_LOCAL(_)
            | Instruction::STORE_LOCAL(_) => 1 + 2,
            Instruction::JMP(_) | Instruction::JMP_IF_TRUE(_) | Instruction::JMP_IF_FALSE(_) => {
                1 + 8
            }
            Instruction::CALL(_) => 1 + 4,
            _ => 1,
        }
    }

    fn finalize(&self) -> Vec<u8> {
        let mut output = Vec::new();

        let type_table = self.generate_type_table();
        let const_pool = self.generate_const_pool();
        let (bytecode, label_positions) = self.generate_bytecode();
        let function_table = self.generate_function_table(&label_positions);

        output.extend(MAGIC_NUMBER);
        output.extend(VERSION.to_be_bytes());
        output.extend(0x0000u16.to_be_bytes());

        let header_size = 33u32; // 5 + 2 + 2 + 4 + 4 + 4 + 4 + 4 + 4
        let type_table_offset = header_size;
        let const_pool_offset = type_table_offset + type_table.len() as u32;
        let function_table_offset = const_pool_offset + const_pool.len() as u32;
        let bytecode_offset = function_table_offset + function_table.len() as u32;

        output.extend((self.main_index.0).to_be_bytes());

        output.extend(type_table_offset.to_be_bytes());
        output.extend(const_pool_offset.to_be_bytes());
        output.extend(function_table_offset.to_be_bytes());
        output.extend(bytecode_offset.to_be_bytes());

        output.extend((bytecode.len() as u32).to_be_bytes());

        output.extend(type_table);
        output.extend(const_pool);
        output.extend(function_table);
        output.extend(bytecode);

        output
    }

    pub fn compile(ast: &Program, analysis: RuntimeAnalysisResult) -> Vec<u8> {
        let mut s = Self::new(analysis);
        ast.generate(&mut s);
        s.finalize()
    }
}
