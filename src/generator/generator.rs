use crate::{
    parser::{expr::Expr, node::NodeId, program::Program},
    semantic_analyzer::{
        const_pool::ConstIndex, function_table::FuncIndex, resolvers::expr_resolver::ExprResolver,
        result::RuntimeAnalysisResult,
    },
};

use super::{
    compile_trait::Generate,
    expr_generators::expr_gen::ExprGenerator,
    instruction::{Instruction, LocalAddress},
};

static PREALLOC_CODE_BUFFER: usize = 4096;

pub struct Generator {
    code: Vec<u8>,
    analysis_result: RuntimeAnalysisResult,
}

impl Generator {
    fn new(analysis: RuntimeAnalysisResult) -> Self {
        Self {
            code: Vec::with_capacity(PREALLOC_CODE_BUFFER),
            analysis_result: analysis,
        }
    }

    pub fn gen<G: Generate>(&mut self, thing: &G) {
        thing.generate(self);
    }

    pub fn gen_expr(&mut self, expr: &Expr) {
        ExprGenerator::generate(expr, self);
    }

    pub fn push_instruction(&mut self, instruction: Instruction) {
        instruction.push_instruction(&mut self.code);
    }

    pub fn current_position(&self) -> usize {
        self.code.len()
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

    pub fn compile(ast: &Program, analysis: RuntimeAnalysisResult) -> Vec<u8> {
        todo!()
    }
}
