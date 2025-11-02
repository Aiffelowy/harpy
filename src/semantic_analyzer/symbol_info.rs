use crate::parser::types::Type;

#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub ttype: Type,
    pub initialized: bool,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub params: Vec<Type>,
    pub return_type: Type,
}

#[derive(Debug, Clone)]
pub enum SymbolInfoKind {
    Function(FunctionInfo),
    Variable(VariableInfo),
}

impl SymbolInfoKind {
    pub fn get_type(&self) -> &Type {
        match self {
            Self::Function(f) => &f.return_type,
            Self::Variable(v) => &v.ttype,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub kind: SymbolInfoKind,
    pub index: usize,
    pub ref_count: usize,
}

impl SymbolInfo {
    pub fn new(kind: SymbolInfoKind, index: usize) -> Self {
        Self {
            kind,
            index,
            ref_count: 0,
        }
    }

    pub fn infer_type(&mut self, ttype: Type) {
        match &mut self.kind {
            SymbolInfoKind::Function(_) => (),
            SymbolInfoKind::Variable(ref mut v) => v.ttype = ttype,
        }
    }
}
