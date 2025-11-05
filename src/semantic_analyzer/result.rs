use crate::extensions::SymbolInfoRefExt;
use std::collections::HashMap;

use crate::{
    aliases::{NodeInfo, Result, ScopeRc},
    parser::node::NodeId,
};

use super::{
    const_pool::ConstPool,
    scope::Scope,
    symbol_info::RuntimeSymbolInfo,
    type_table::{RuntimeTypeTable, TypeTable},
};

#[derive(Debug)]
pub struct AnalysisResult {
    pub scope_tree: ScopeRc,
    pub node_info: NodeInfo,
    pub type_table: TypeTable,
    pub constants: ConstPool,
}

impl AnalysisResult {
    pub(in crate::semantic_analyzer) fn new() -> Self {
        let root = Scope::new(super::scope::ScopeKind::Global, None);
        let root = ScopeRc::new(root.into());
        Self {
            scope_tree: root,
            node_info: NodeInfo::new(),
            type_table: TypeTable::new(),
            constants: ConstPool::new(),
        }
    }

    pub fn into_runtime(self) -> Result<RuntimeAnalysisResult> {
        let type_table = self.type_table.into_runtime()?;
        let node_info = self
            .node_info
            .iter()
            .map(|(k, v)| (*k, v.get().into_runtime(&type_table)))
            .collect();

        Ok(RuntimeAnalysisResult {
            constants: self.constants,
            type_table,
            node_info,
        })
    }
}

#[derive(Debug)]
pub struct RuntimeAnalysisResult {
    pub node_info: HashMap<NodeId, RuntimeSymbolInfo>,
    pub type_table: RuntimeTypeTable,
    pub constants: ConstPool,
}
