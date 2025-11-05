use std::fmt::Display;

use crate::{
    aliases::{Result, TypeInfoRc},
    parser::{
        node::NodeId,
        types::{RuntimeType, Type},
    },
};

use super::type_table::{RuntimeTypeIndex, RuntimeTypeTable, TypeIndex};

macro_rules! define_runtime_enum {
    (
        $(#[$enum_meta:meta])*
        $vis:vis enum ($name:ident, $runtime_name:ident) {
            $(
                $variant:ident($inner:ty, $inner_runtime:ty),
            )*
        }
    ) => {
        $(#[$enum_meta])*
        $vis enum $name {
            $(
                $variant($inner),
            )*
        }

        $(#[$enum_meta])*
        $vis enum $runtime_name {
            $(
                $variant($inner_runtime),
            )*
        }
    };
}

#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub ttype: Type,
    pub size: usize,
    pub idx: TypeIndex,
}

#[derive(Debug, Clone)]
pub struct RuntimeTypeInfo {
    pub ttype: RuntimeType,
    pub size: usize,
}

#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub ttype: TypeInfoRc,
    pub initialized: bool,
}

#[derive(Debug, Clone)]
pub struct RuntimeVariableInfo {
    pub ttype: RuntimeTypeIndex,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub params: Vec<ParamInfo>,
    pub return_type: TypeInfoRc,
    pub locals: Vec<TypeInfoRc>,
}

#[derive(Debug, Clone)]
pub struct RuntimeFunctionInfo {
    pub params: Vec<RuntimeParamInfo>,
    pub return_type: RuntimeTypeIndex,
    pub locals: Vec<RuntimeTypeIndex>,
}

#[derive(Debug, Clone)]
pub struct ExprInfo {
    pub ttype: TypeInfoRc,
}

#[derive(Debug, Clone)]
pub struct RuntimeExprInfo {
    pub ttype: RuntimeTypeIndex,
}

#[derive(Debug, Clone)]
pub struct ParamInfo {
    pub ttype: TypeInfoRc,
}

#[derive(Debug, Clone)]
pub struct RuntimeParamInfo {
    pub ttype: RuntimeTypeIndex,
}

define_runtime_enum! {
#[derive(Debug, Clone)]
pub enum (SymbolInfoKind, RuntimeSymbolInfoKind) {
    Function(FunctionInfo, RuntimeFunctionInfo),
    Variable(VariableInfo, RuntimeVariableInfo),
    Param(ParamInfo, RuntimeParamInfo),
    Expr(ExprInfo, RuntimeExprInfo),
}}

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

    pub fn into_runtime(&self) -> Result<RuntimeTypeInfo> {
        Ok(RuntimeTypeInfo {
            ttype: self.ttype.to_runtime()?,
            size: self.size,
        })
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

    fn function_to_runtime(
        info: &FunctionInfo,
        type_table: &RuntimeTypeTable,
    ) -> RuntimeFunctionInfo {
        let params = info
            .params
            .iter()
            .map(|param| RuntimeParamInfo {
                ttype: type_table.get_mapping(&param.ttype.idx),
            })
            .collect::<Vec<RuntimeParamInfo>>();
        let locals = info
            .locals
            .iter()
            .map(|local| type_table.get_mapping(&local.idx))
            .collect::<Vec<RuntimeTypeIndex>>();
        RuntimeFunctionInfo {
            params,
            locals,
            return_type: type_table.get_mapping(&info.return_type.idx),
        }
    }

    pub(in crate::semantic_analyzer) fn into_runtime(
        &self,
        type_table: &RuntimeTypeTable,
    ) -> RuntimeSymbolInfoKind {
        if let Self::Function(ref f) = &self {
            return RuntimeSymbolInfoKind::Function(Self::function_to_runtime(f, type_table));
        }

        let idx = self.get_type().idx;
        let ttype = type_table.get_mapping(&idx);

        match self {
            Self::Function(_) => unreachable!(),
            Self::Expr(_) => RuntimeSymbolInfoKind::Expr(RuntimeExprInfo { ttype }),
            Self::Variable(_) => RuntimeSymbolInfoKind::Variable(RuntimeVariableInfo { ttype }),
            Self::Param(_) => RuntimeSymbolInfoKind::Param(RuntimeParamInfo { ttype }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub kind: SymbolInfoKind,
    pub ref_count: usize,
    pub node_id: NodeId,
}

#[derive(Debug, Clone)]
pub struct RuntimeSymbolInfo {
    pub kind: RuntimeSymbolInfoKind,
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

    pub(in crate::semantic_analyzer) fn into_runtime(
        &self,
        type_table: &RuntimeTypeTable,
    ) -> RuntimeSymbolInfo {
        RuntimeSymbolInfo {
            kind: self.kind.into_runtime(type_table),
            node_id: self.node_id,
        }
    }
}
