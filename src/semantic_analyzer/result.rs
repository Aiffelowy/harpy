use crate::aliases::{NodeInfo, ScopeRc};

use super::{const_pool::ConstPool, scope::Scope, type_table::TypeTable};

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
}
