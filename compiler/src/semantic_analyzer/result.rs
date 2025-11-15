use std::collections::HashMap;

use crate::{
    aliases::{NodeInfo, ScopeRc},
    err::HarpyError,
    extensions::SymbolInfoRefExt,
    generator::instruction::LocalAddress,
    parser::node::NodeId,
};

use super::{
    const_pool::{ConstPool, RuntimeConstPool},
    function_table::{FunctionTable, RuntimeFunctionTable},
    scope::{Depth, Scope},
    symbol_info::SymbolInfoKind,
    type_table::{RuntimeConversionTypeTable, RuntimeTypeIndex, RuntimeTypeTable, TypeTable},
};

fn node_info_to_expr_map(
    node_info: &NodeInfo,
    type_table: &RuntimeConversionTypeTable,
) -> HashMap<NodeId, RuntimeTypeIndex> {
    let mut expr_map = HashMap::new();
    for (id, info) in node_info.iter() {
        let info = info.get();
        if let SymbolInfoKind::Expr = info.kind {
            let ty_id = type_table.get_mapping(&info.ty.idx);
            expr_map.insert(*id, ty_id);
        }
    }

    expr_map
}

#[derive(Debug)]
pub struct AnalysisResult {
    pub scope_tree: ScopeRc,
    pub node_info: NodeInfo,
    pub type_table: TypeTable,
    pub constants: ConstPool,
    pub function_table: FunctionTable,
    pub locals_map: HashMap<NodeId, LocalAddress>,
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
            locals_map: HashMap::new(),
        }
    }

    pub fn into_runtime(self) -> std::result::Result<RuntimeAnalysisResult, Vec<HarpyError>> {
        let type_table = self.type_table.into_conversion().map_err(|e| vec![e])?;
        let constants = self.constants.to_runtime(&type_table);
        let function_table = self.function_table.into_runtime(&type_table)?;
        let expr_map = node_info_to_expr_map(&self.node_info, &type_table);

        Ok(RuntimeAnalysisResult {
            constants,
            type_table: type_table.into_runtime(),
            function_table,
            locals_map: self.locals_map,
            expr_map,
        })
    }
}

#[derive(Debug)]
pub struct RuntimeAnalysisResult {
    pub type_table: RuntimeTypeTable,
    pub constants: RuntimeConstPool,
    pub function_table: RuntimeFunctionTable,
    pub locals_map: HashMap<NodeId, LocalAddress>,
    pub expr_map: HashMap<NodeId, RuntimeTypeIndex>,
}
