use crate::aliases::{NodeInfo, Result, ScopeRc};

use super::{
    const_pool::{ConstPool, RuntimeConstPool},
    function_table::{FunctionTable, RuntimeFunctionTable},
    scope::{Depth, Scope},
    type_table::{RuntimeTypeTable, TypeTable},
};

#[derive(Debug)]
pub struct AnalysisResult {
    pub scope_tree: ScopeRc,
    pub node_info: NodeInfo,
    pub type_table: TypeTable,
    pub constants: ConstPool,
    pub function_table: FunctionTable,
}

impl AnalysisResult {
    pub(in crate::semantic_analyzer) fn new() -> Self {
        let root = Scope::new(super::scope::ScopeKind::Global, None, Depth(0));
        let root = ScopeRc::new(root.into());
        Self {
            scope_tree: root,
            node_info: NodeInfo::new(),
            type_table: TypeTable::new(),
            constants: ConstPool::new(),
            function_table: FunctionTable::new(),
        }
    }

    pub fn into_runtime(self) -> Result<RuntimeAnalysisResult> {
        let type_table = self.type_table.into_conversion()?;
        let constants = self.constants.to_runtime(&type_table);
        let function_table = self.function_table.into_runtime(&type_table);

        Ok(RuntimeAnalysisResult {
            constants,
            type_table: type_table.into_runtime(),
            function_table,
        })
    }
}

#[derive(Debug)]
pub struct RuntimeAnalysisResult {
    pub type_table: RuntimeTypeTable,
    pub constants: RuntimeConstPool,
    pub function_table: RuntimeFunctionTable,
}
