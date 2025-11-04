use crate::parser::{node::NodeId, types::Type};

#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub ttype: Type,
    pub initialized: bool,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub params: Vec<Type>,
    pub return_type: Type,
    pub locals: Vec<Type>,
}

#[derive(Debug, Clone)]
pub struct ExprInfo {
    pub ttype: Type,
}

#[derive(Debug, Clone)]
pub enum SymbolInfoKind {
    Function(FunctionInfo),
    Variable(VariableInfo),
    Expr(ExprInfo),
}

impl SymbolInfoKind {
    pub fn get_type(&self) -> &Type {
        match self {
            Self::Function(f) => &f.return_type,
            Self::Variable(v) => &v.ttype,
            Self::Expr(e) => &e.ttype,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub kind: SymbolInfoKind,
    pub ref_count: usize,
    pub node_id: NodeId,
}

impl SymbolInfo {
    pub fn new(kind: SymbolInfoKind, node_id: NodeId) -> Self {
        Self {
            kind,
            ref_count: 0,
            node_id,
        }
    }

    pub fn infer_type(&mut self, ttype: Type) {
        match &mut self.kind {
            SymbolInfoKind::Function(_) => (),
            SymbolInfoKind::Variable(ref mut v) => v.ttype = ttype,
            SymbolInfoKind::Expr(_) => (),
        }
    }
}
