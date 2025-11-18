use std::collections::HashMap;

use crate::{
    aliases::{SymbolInfoRef},
    err::HarpyError,
    extensions::SymbolInfoRefExt,
    lexer::tokens::Ident,
    parser::{
        node::{Node, NodeId},
        types::TypeInner,
    },
    generator::instruction::GlobalAddress,
};

use super::{
    err::SemanticError,
    symbol_info::SymbolInfoKind,
    type_table::{RuntimeConversionTypeTable, RuntimeTypeIndex},
};

#[derive(Debug)]
pub struct RuntimeGlobalInfo {
    pub type_index: RuntimeTypeIndex,
}

#[derive(Debug)]
pub struct GlobalTable {
    pool: Vec<SymbolInfoRef>,
    node_map: HashMap<NodeId, GlobalAddress>,
}

impl GlobalTable {
    pub fn new() -> Self {
        Self {
            pool: vec![],
            node_map: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &Node<Ident>, info: SymbolInfoRef) -> GlobalAddress {
        let idx = GlobalAddress(self.pool.len().try_into().unwrap());
        self.pool.push(info.clone());
        self.node_map.insert(name.id(), idx);
        idx
    }

    pub fn map_node_to_global(&mut self, node_id: NodeId, global_addr: GlobalAddress) {
        self.node_map.insert(node_id, global_addr);
    }

    pub fn get(&self, idx: GlobalAddress) -> SymbolInfoRef {
        self.pool[idx.0 as usize].clone()
    }

    pub fn get_mapping(&self, node_id: NodeId) -> GlobalAddress {
        self.node_map[&node_id]
    }

    pub fn get_mapping_opt(&self, node_id: NodeId) -> Option<GlobalAddress> {
        self.node_map.get(&node_id).copied()
    }

    pub fn contains_node(&self, node_id: NodeId) -> bool {
        self.node_map.contains_key(&node_id)
    }

    pub(in crate::semantic_analyzer) fn into_runtime(
        self,
        type_table: &RuntimeConversionTypeTable,
    ) -> std::result::Result<RuntimeGlobalTable, Vec<HarpyError>> {
        let mut pool = Vec::with_capacity(self.pool.len());
        let mut error_pool = vec![];
        
        for info in self.pool {
            let info = info.get();
            if let SymbolInfoKind::Global(_) = &info.kind {
                let ty = &info.ty;
                if ty.inner == TypeInner::Unknown {
                    error_pool.push(HarpyError::new(
                        crate::err::HarpyErrorKind::SemanticError(
                            SemanticError::CantInferType,
                        ),
                        info.span,
                    ));
                    continue;
                }
                
                let type_index = type_table.get_mapping(&ty.idx);
                pool.push(RuntimeGlobalInfo { type_index });
            }
        }

        if !error_pool.is_empty() {
            return Err(error_pool);
        }

        Ok(RuntimeGlobalTable { 
            pool,
            node_map: self.node_map,
        })
    }
}

#[derive(Debug)]
pub struct RuntimeGlobalTable {
    pool: Vec<RuntimeGlobalInfo>,
    node_map: HashMap<NodeId, GlobalAddress>,
}

impl RuntimeGlobalTable {
    pub fn get(&self, idx: GlobalAddress) -> &RuntimeGlobalInfo {
        &self.pool[idx.0 as usize]
    }

    pub fn get_mapping(&self, node_id: NodeId) -> GlobalAddress {
        self.node_map[&node_id]
    }

    pub fn contains_node(&self, node_id: NodeId) -> bool {
        self.node_map.contains_key(&node_id)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, RuntimeGlobalInfo> {
        self.pool.iter()
    }
}
