use crate::{
    analyzer::{symbols::symbols::SymbolId, types::types::TypeId},
    lexer::span::Span,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(pub usize);

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub id: Option<FunctionId>,
    pub signature: TypeId,
    pub params: Vec<SymbolId>,
    pub span: Span,

    pub locals: Vec<SymbolId>,
}

#[derive(Debug, Default)]
pub struct FunctionTable {
    pub functions: Vec<FunctionDef>,
}

impl FunctionTable {
    pub fn register(&mut self, mut def: FunctionDef) -> FunctionId {
        let id = FunctionId(self.functions.len());
        def.id = Some(id);
        self.functions.push(def);

        id
    }

    pub fn get(&self, id: FunctionId) -> &FunctionDef {
        &self.functions[id.0]
    }
}
