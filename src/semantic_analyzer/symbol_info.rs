use std::fmt::Display;

use crate::{
    aliases::TypeInfoRc,
    parser::{node::NodeId, types::Type},
};

#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub ttype: Type,
    pub size: usize,
}

#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub ttype: TypeInfoRc,
    pub initialized: bool,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub params: Vec<TypeInfoRc>,
    pub return_type: TypeInfoRc,
    pub locals: Vec<TypeInfoRc>,
}

#[derive(Debug, Clone)]
pub struct ExprInfo {
    pub ttype: TypeInfoRc,
}

#[derive(Debug, Clone)]
pub struct ParamInfo {
    pub ttype: TypeInfoRc,
}

#[derive(Debug, Clone)]
pub enum SymbolInfoKind {
    Function(FunctionInfo),
    Variable(VariableInfo),
    Param(ParamInfo),
    Expr(ExprInfo),
}

impl TypeInfo {
    pub fn compatible(&self, other: &Type) -> bool {
        self.ttype.compatible(other)
    }

    pub fn strict_compatible(&self, other: &Type) -> bool {
        self.ttype.strict_compatible(other)
    }

    pub fn assign_compatible(&self, other: &Type) -> bool {
        self.ttype.assign_compatible(other)
    }

    pub fn param_compatible(&self, other: &Type) -> bool {
        self.ttype.param_compatible(other)
    }

    pub fn return_compatible(&self, other: &Type) -> bool {
        self.ttype.return_compatible(other)
    }
}

impl Display for TypeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ttype)
    }
}

impl FunctionInfo {
    pub fn new(return_type: TypeInfoRc) -> Self {
        Self {
            params: vec![],
            locals: vec![],
            return_type,
        }
    }
}

impl SymbolInfoKind {
    pub fn get_type(&self) -> &TypeInfoRc {
        match self {
            Self::Function(f) => &f.return_type,
            Self::Variable(v) => &v.ttype,
            Self::Expr(e) => &e.ttype,
            Self::Param(p) => &p.ttype,
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

    pub fn infer_type(&mut self, ttype: &TypeInfoRc) {
        match &mut self.kind {
            SymbolInfoKind::Function(_) => (),
            SymbolInfoKind::Variable(ref mut v) => v.ttype = ttype.clone(),
            SymbolInfoKind::Param(_) => (),
            SymbolInfoKind::Expr(_) => (),
        }
    }
}
