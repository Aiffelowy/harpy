use std::{fmt::Display, ops::Deref};

use crate::{
    aliases::{Result, SymbolInfoRef, TypeInfoRc},
    err::HarpyError,
    lexer::span::Span,
    parser::{
        node::NodeId,
        types::{RuntimeType, Type, TypeInner},
    },
};

use super::{
    const_pool::ConstIndex,
    scope::Depth,
    type_table::{RuntimeConversionTypeTable, RuntimeTypeIndex, TypeIndex},
};

impl Type {
    pub fn to_runtime(&self, type_table: &RuntimeConversionTypeTable) -> Result<RuntimeType> {
        let new = match &self.inner {
            TypeInner::Void => RuntimeType::Void,
            TypeInner::Unknown => {
                return HarpyError::semantic(
                    crate::semantic_analyzer::err::SemanticError::UnresolvedType,
                    Span::default(),
                )
            }
            TypeInner::Ref(t) => {
                RuntimeType::Ref(type_table.get_mapping(&type_table.get_type_index(t)))
            }
            TypeInner::Boxed(t) => {
                RuntimeType::Boxed(type_table.get_mapping(&type_table.get_type_index(t)))
            }
            TypeInner::Base(b) => RuntimeType::Base(b.clone()),
        };

        Ok(new)
    }
}

#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub ttype: Type,
    pub size: u8,
    pub idx: TypeIndex,
}

#[derive(Debug, Clone)]
pub struct RuntimeTypeInfo {
    pub ttype: RuntimeType,
    pub size: u8,
}

#[derive(Debug, Clone)]
pub struct BorrowInfo {
    pub depth: Depth,
    pub original: SymbolInfoRef,
    pub borrow_span: Span,
}

#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub initialized: bool,
    pub mutably_borrowed: bool,
    pub immutably_borrowed_count: usize,
}

impl VariableInfo {
    pub fn new() -> Self {
        Self {
            initialized: false,
            mutably_borrowed: false,
            immutably_borrowed_count: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub params: Vec<TypeInfoRc>,
    pub locals: Vec<SymbolInfoRef>,
}

#[derive(Debug, Clone)]
pub struct RuntimeFunctionInfo {
    pub params: Vec<RuntimeTypeIndex>,
    pub locals: Vec<RuntimeTypeIndex>,
    pub return_type: RuntimeTypeIndex,
}

#[derive(Debug, Clone)]
pub struct LiteralInfo {
    pub const_idx: ConstIndex,
}

#[derive(Debug, Clone)]
pub struct RuntimeLiteralInfo {
    pub const_idx: ConstIndex,
}

#[derive(Debug, Clone)]
pub enum SymbolInfoKind {
    Function(FunctionInfo),
    Variable(VariableInfo),
    Literal(LiteralInfo),
    Param,
    Expr,
}

impl TypeInfo {
    pub fn into_runtime(&self, type_table: &RuntimeConversionTypeTable) -> Result<RuntimeTypeInfo> {
        Ok(RuntimeTypeInfo {
            ttype: self.ttype.to_runtime(type_table)?,
            size: self.size,
        })
    }
}

impl Deref for TypeInfo {
    type Target = Type;
    fn deref(&self) -> &Self::Target {
        &self.ttype
    }
}

impl Display for TypeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ttype)
    }
}

impl FunctionInfo {
    pub fn new() -> Self {
        Self {
            params: vec![],
            locals: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub ty: TypeInfoRc,
    pub kind: SymbolInfoKind,
    pub ref_count: usize,
    pub node_id: NodeId,
    pub scope_depth: Depth,
    pub span: Span,
}

impl SymbolInfo {
    pub fn new(
        ty: TypeInfoRc,
        kind: SymbolInfoKind,
        node_id: NodeId,
        scope_depth: Depth,
        span: Span,
    ) -> Self {
        Self {
            ty,
            kind,
            ref_count: 0,
            node_id,
            scope_depth,
            span,
        }
    }

    pub fn infer_type(&mut self, ttype: &TypeInfoRc) {
        self.ty = TypeInfoRc::new(TypeInfo {
            ttype: Type {
                mutable: self.ty.mutable,
                inner: ttype.ttype.inner.clone(),
            },
            size: ttype.size,
            idx: ttype.idx,
        })
    }
}

impl Display for SymbolInfoKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Expr => "expression",
            Self::Param => "parameter",
            Self::Literal(_) => "literal",
            Self::Function(_) => "function",
            Self::Variable(_) => "variable",
        };

        write!(f, "{s}")
    }
}
