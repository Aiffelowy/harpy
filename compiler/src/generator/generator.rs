use std::collections::HashMap;

use crate::{
    aliases::{MAGIC_NUMBER, VERSION},
    lexer::tokens::Lit,
    parser::{expr::Expr, node::NodeId, program::Program, types::runtime::RuntimeType},
    semantic_analyzer::{
        const_pool::ConstIndex, function_table::FuncIndex, result::RuntimeAnalysisResult,
        type_table::RuntimeTypeIndex,
    },
};

use super::{
    compile_trait::Generate,
    expr_generators::expr_gen::ExprGenerator,
    instruction::{GlobalAddress, Instruction, Label, LocalAddress},
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

    next_label: u64,
}

impl Generator {
    fn new(analysis: RuntimeAnalysisResult) -> Self {
        Self {
            code: Vec::with_capacity(PREALLOC_CODE_BUFFER),
            function_entry_points: HashMap::new(),
            analysis_result: analysis,
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

    pub fn get_global_mapping(&self, id: NodeId) -> GlobalAddress {
        self.analysis_result.global_table.get_mapping(id)
    }

    pub fn is_global(&self, id: NodeId) -> bool {
        self.analysis_result.global_table.contains_node(id)
    }

    pub fn is_local(&self, id: NodeId) -> bool {
        self.analysis_result.locals_map.contains_key(&id)
    }

    pub fn get_call_mapping(&self, id: NodeId) -> FuncIndex {
        self.analysis_result.function_table.get_mapping(id)
    }

    pub fn get_main_index(&self) -> FuncIndex {
        self.analysis_result.main_id
    }

    pub fn get_const_mapping(&self, id: NodeId) -> ConstIndex {
        self.analysis_result.constants.get_mapping(id)
    }

    pub fn get_expr_type(&self, id: NodeId) -> RuntimeTypeIndex {
        self.analysis_result.expr_map[&id]
    }

    pub fn get_type_info(
        &self,
        type_idx: RuntimeTypeIndex,
    ) -> &crate::semantic_analyzer::symbol_info::RuntimeTypeInfo {
        self.analysis_result.type_table.get(type_idx)
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

    fn generate_global_table(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();

        for global in self.analysis_result.global_table.iter() {
            data.extend(global.type_index.0.to_be_bytes());
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
                    position += self.instruction_size(instr);
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
                    Instruction::LOAD_GLOBAL(addr) | Instruction::STORE_GLOBAL(addr) => {
                        data.extend(addr.0.to_be_bytes());
                    }
                    Instruction::BOX_ALLOC(rti) => {
                        data.extend(rti.0.to_be_bytes());
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
            | Instruction::STORE_LOCAL(_)
            | Instruction::LOAD_GLOBAL(_)
            | Instruction::STORE_GLOBAL(_) => 1 + 2,
            Instruction::BOX_ALLOC(_) => 1 + 4,
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
        let global_table = self.generate_global_table();
        let const_pool = self.generate_const_pool();
        let (bytecode, label_positions) = self.generate_bytecode();
        let function_table = self.generate_function_table(&label_positions);

        output.extend(MAGIC_NUMBER);
        output.extend(VERSION.to_be_bytes());
        output.extend(0x0000u16.to_be_bytes());

        let header_size = 37u32; // 5 + 2 + 2 + 4 + 4 + 4 + 4 + 4 + 4 + 4
        let type_table_offset = header_size;
        let global_table_offset = type_table_offset + type_table.len() as u32;
        let const_pool_offset = global_table_offset + global_table.len() as u32;
        let function_table_offset = const_pool_offset + const_pool.len() as u32;
        let bytecode_offset = function_table_offset + function_table.len() as u32;

        output.extend((self.analysis_result.main_id.0).to_be_bytes());

        output.extend(type_table_offset.to_be_bytes());
        output.extend(global_table_offset.to_be_bytes());
        output.extend(const_pool_offset.to_be_bytes());
        output.extend(function_table_offset.to_be_bytes());
        output.extend(bytecode_offset.to_be_bytes());

        output.extend((bytecode.len() as u32).to_be_bytes());

        output.extend(type_table);
        output.extend(global_table);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        lexer::Lexer, parser::parser::Parser, semantic_analyzer::analyzer::Analyzer,
        source::SourceFile,
    };
    use std::io::Cursor;

    fn create_generator_with_ast(
        source_code: &str,
    ) -> Result<Generator, Vec<crate::err::HarpyError>> {
        let reader = Cursor::new(source_code);
        let source = SourceFile::new(reader).map_err(|e| vec![e])?;

        let lexer = Lexer::new(&source).map_err(|e| vec![e])?;
        let parser = Parser::new(lexer);

        let ast = parser.build_ast()?;
        let analysis_result = Analyzer::analyze(&ast)?.into_runtime()?;

        let mut generator = Generator::new(analysis_result);
        ast.generate(&mut generator);
        Ok(generator)
    }

    #[test]
    fn test_generate_empty_main() {
        let source = "fn main() {}";
        let generator = create_generator_with_ast(source).unwrap();

        assert!(!generator.code.is_empty());
        assert!(generator
            .code
            .iter()
            .any(|node| matches!(node, BytecodeNode::Instruction(Instruction::RET))));
    }

    #[test]
    fn test_generate_simple_return() {
        let source = "fn main() -> int { return 42; }";
        let generator = create_generator_with_ast(source).unwrap();

        let has_load_const = generator
            .code
            .iter()
            .any(|node| matches!(node, BytecodeNode::Instruction(Instruction::LOAD_CONST(_))));
        let has_ret = generator
            .code
            .iter()
            .any(|node| matches!(node, BytecodeNode::Instruction(Instruction::RET)));

        assert!(
            has_load_const,
            "Should have LOAD_CONST instruction for literal 42"
        );
        assert!(has_ret, "Should have RET instruction");
    }

    #[test]
    fn test_generate_function_with_params() {
        let source = "fn add(a: int, b: int) -> int { return a + b; } fn main() {}";
        let generator = create_generator_with_ast(source).unwrap();

        let has_add = generator
            .code
            .iter()
            .any(|node| matches!(node, BytecodeNode::Instruction(Instruction::ADD)));
        let has_ret = generator
            .code
            .iter()
            .any(|node| matches!(node, BytecodeNode::Instruction(Instruction::RET)));

        assert!(has_add, "Should have ADD instruction for a + b");
        assert!(has_ret, "Should have RET instruction");
    }

    #[test]
    fn test_generate_labels() {
        let source = "fn main() {} fn helper() {}";
        let generator = create_generator_with_ast(source).unwrap();

        let label_count = generator
            .code
            .iter()
            .filter(|node| matches!(node, BytecodeNode::Label(_)))
            .count();

        assert!(
            label_count >= 2,
            "Should have at least 2 labels for 2 functions"
        );
    }

    #[test]
    fn test_generate_global_access() {
        let source = "global x: int = 42; fn main() -> int { return x; }";
        let generator = create_generator_with_ast(source).unwrap();

        let has_load_global = generator
            .code
            .iter()
            .any(|node| matches!(node, BytecodeNode::Instruction(Instruction::LOAD_GLOBAL(_))));

        assert!(
            has_load_global,
            "Should have LOAD_GLOBAL instruction for accessing global x"
        );
    }

    #[test]
    fn test_generate_switch_statement() {
        let source = r#"
            fn main(x: int) -> int {
                switch x {
                    1 -> return 10;
                    2 -> return 20;
                }
                return 0;
            }
        "#;
        let generator = create_generator_with_ast(source).unwrap();

        let has_dup = generator
            .code
            .iter()
            .any(|node| matches!(node, BytecodeNode::Instruction(Instruction::DUP)));
        let has_eq = generator
            .code
            .iter()
            .any(|node| matches!(node, BytecodeNode::Instruction(Instruction::EQ)));
        let has_jmp_if_true = generator
            .code
            .iter()
            .any(|node| matches!(node, BytecodeNode::Instruction(Instruction::JMP_IF_TRUE(_))));
        let has_pop = generator
            .code
            .iter()
            .any(|node| matches!(node, BytecodeNode::Instruction(Instruction::POP)));

        assert!(
            has_dup,
            "Switch should use DUP instruction to cache switch value"
        );
        assert!(
            has_eq,
            "Switch should use EQ instruction for case comparison"
        );
        assert!(
            has_jmp_if_true,
            "Switch should use JMP_IF_TRUE for case branches"
        );
        assert!(has_pop, "Switch should use POP to clean up stack");
    }

    #[test]
    fn test_bytecode_compilation_full() {
        let generator = create_generator_with_ast("fn main() {}").unwrap();
        let bytecode = generator.finalize();

        assert!(!bytecode.is_empty());
        assert!(bytecode.starts_with(&crate::aliases::MAGIC_NUMBER));

        let version_bytes = &bytecode[5..7];
        let version = u16::from_be_bytes([version_bytes[0], version_bytes[1]]);
        assert_eq!(version, crate::aliases::VERSION);
    }

    #[test]
    fn test_bytecode_size() {
        let generator = create_generator_with_ast("fn main() {}").unwrap();
        let bytecode = generator.finalize();
        let bytecode_offset_pos = 5 + 2 + 2 + 4 + 4 + 4 + 4 + 4;
        let bytecode_offset = u32::from_be_bytes([
            bytecode[bytecode_offset_pos],
            bytecode[bytecode_offset_pos + 1],
            bytecode[bytecode_offset_pos + 2],
            bytecode[bytecode_offset_pos + 3],
        ]) as usize;
        let bytecode_size_pos = bytecode_offset_pos + 4;
        let bytecode_size = u32::from_be_bytes([
            bytecode[bytecode_size_pos],
            bytecode[bytecode_size_pos + 1],
            bytecode[bytecode_size_pos + 2],
            bytecode[bytecode_size_pos + 3],
        ]) as usize;
        let bytecode_chunk = &bytecode[bytecode_offset..];
        assert_eq!(bytecode_chunk.len(), bytecode_size)
    }

    #[test]
    fn test_exact_instruction_sequence() {
        let generator = create_generator_with_ast(
            "global x:int = 1; fn add(a:int, b:int) -> int { return a+b; }  fn main() -> int { return add(x,4); }",
        )
        .unwrap();
        let instructions: Vec<_> = generator
            .code
            .into_iter()
            .filter_map(|node| match node {
                BytecodeNode::Instruction(i) => Some(i),
                BytecodeNode::Label(_) => None,
            })
            .collect();
        let expected = vec![
            Instruction::LOAD_CONST(ConstIndex(1)),
            Instruction::STORE_GLOBAL(GlobalAddress(0)),
            Instruction::CALL(FuncIndex(1)),
            Instruction::HALT,
            Instruction::LOAD_LOCAL(LocalAddress(0)),
            Instruction::LOAD_LOCAL(LocalAddress(1)),
            Instruction::ADD,
            Instruction::RET,
            Instruction::LOAD_GLOBAL(GlobalAddress(0)),
            Instruction::LOAD_CONST(ConstIndex(2)),
            Instruction::CALL(FuncIndex(0)),
            Instruction::RET,
        ];

        assert_eq!(instructions, expected)
    }
}
