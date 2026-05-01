use crate::{
    analyzer::{
        analyzer::{Analyzer, Fallback},
        tables::symbol_table::SymbolId,
        types::types::TypeId,
    },
    lexer::span::Span,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(pub usize);

impl FunctionId {
    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }
}

impl Fallback<FunctionId> for Analyzer {
    fn fallback(&self) -> FunctionId {
        FunctionId(0)
    }
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub id: Option<FunctionId>,
    pub signature: TypeId,
    pub params: Vec<SymbolId>,
    pub span: Span,

    pub locals: Vec<SymbolId>,
}

impl FunctionDef {
    pub fn skeleton(name: String, signature: TypeId, span: Span) -> Self {
        FunctionDef {
            name,
            id: None,
            signature,
            params: Vec::new(),
            span,
            locals: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct FunctionTable {
    pub functions: Vec<FunctionDef>,
}

impl Default for FunctionTable {
    fn default() -> Self {
        let mut s = Self {
            functions: Vec::new(),
        };
        let dummy = FunctionDef::skeleton("<unknown_fn>".to_owned(), TypeId(0), Span::default());
        s.register(dummy);
        s
    }
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
