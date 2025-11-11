use std::collections::HashMap;

use crate::{
    parser::{expr::Expr, node::NodeId, program::Program},
    semantic_analyzer::{
        const_pool::ConstIndex, function_table::FuncIndex, result::RuntimeAnalysisResult,
    },
};

use super::{
    compile_trait::Generate,
    expr_generators::expr_gen::ExprGenerator,
    instruction::{Instruction, Label, LocalAddress},
};

static PREALLOC_CODE_BUFFER: usize = 4096;

#[derive(Debug, Clone, Copy)]
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

    pub fn get_call_mapping(&self, id: NodeId) -> FuncIndex {
        self.analysis_result.function_table.get_mapping(id)
    }

    pub fn get_const_mapping(&self, id: NodeId) -> ConstIndex {
        self.analysis_result.constants.get_mapping(id)
    }

    pub fn get_function_mapping(&self, id: NodeId) -> FuncIndex {
        self.analysis_result
            .function_table
            .get_function_delc_mapping(id)
    }

    pub fn compile(ast: &Program, analysis: RuntimeAnalysisResult) -> Vec<BytecodeNode> {
        let mut s = Self::new(analysis);
        ast.generate(&mut s);
        s.code
    }
}
