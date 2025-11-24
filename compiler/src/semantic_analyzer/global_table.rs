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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        aliases::{SymbolInfoRef, TypeInfoRc},
        extensions::SymbolInfoRefExt,
        lexer::{span::Span, tokens::Ident},
        parser::{node::Node, types::{Type}},
        semantic_analyzer::{
            symbol_info::{GlobalInfo, SymbolInfo, SymbolInfoKind},
            type_table::{TypeTable},
        },
    };
    use std::{cell::RefCell, rc::Rc};

    fn create_global_symbol(node_id: NodeId, type_info: TypeInfoRc) -> SymbolInfoRef {
        let symbol_info = SymbolInfo {
            kind: SymbolInfoKind::Global(GlobalInfo {}),
            span: Span::default(),
            ty: type_info,
            node_id,
            ref_count: 0,
            scope_depth: crate::semantic_analyzer::scope::Depth(0)
        };
        Rc::new(RefCell::new(symbol_info))
    }


    #[test]
    fn test_global_table_initialization() {
        let table = GlobalTable::new();
        assert_eq!(table.pool.len(), 0);
        assert!(table.node_map.is_empty());
    }

    #[test]
    fn test_global_table_register() {
        let mut table = GlobalTable::new();
        let mut type_table = TypeTable::new();
        
        let ident = Node::dummy(Ident { span: Span::default(), value: "global_var".to_owned() });
        let int_info = type_table.register(&Type::int());
        
        let symbol = create_global_symbol( NodeId(0), int_info );
        let addr = table.register(&ident, symbol.clone());
        
        assert_eq!(addr, GlobalAddress(0));
        assert_eq!(table.pool.len(), 1);
        assert!(table.contains_node(ident.id()));
        
        let retrieved = table.get(addr);
        let ty = (*retrieved.get().ty).ttype.clone();
        assert_eq!(ty, Type::int());
    }

    #[test]
    fn test_global_table_multiple_globals() {
        let mut table = GlobalTable::new();
        let mut type_table = TypeTable::new();
        
        let globals = vec![
            ("x", Type::int()),
            ("y", Type::float()),
            ("flag", Type::bool()),
            ("message", Type::str()),
        ];
        
        let mut addresses = Vec::new();
        for (i, (name, ty)) in globals.iter().enumerate() {
            let ident = Node::dummy_with_id(Ident { span: Span::default(), value: name.to_string() }, NodeId(i));
            let type_info = type_table.register(ty);
            let symbol = create_global_symbol(ident.id(), type_info);
            
            let addr = table.register(&ident, symbol);
            addresses.push((addr, ident.id()));
        }
        
        assert_eq!(table.pool.len(), 4);
        
        for (i, (addr, node_id)) in addresses.iter().enumerate() {
            assert_eq!(*addr, GlobalAddress(i as u16));
            assert_eq!(table.get_mapping(*node_id), *addr);
            assert!(table.contains_node(*node_id));
            
            let symbol = table.get(*addr);
            let ty = (*symbol.get().ty).ttype.clone();
            assert_eq!(ty, globals[i].1);
        }
    }

    #[test]
    fn test_global_table_node_mapping() {
        let mut table = GlobalTable::new();
        let mut type_table = TypeTable::new();
        
        let ident = Node::dummy(Ident { span: Span::default(), value: "test_var".to_owned() });
        let int_info = type_table.register(&Type::int());
        let symbol = create_global_symbol(ident.id(), int_info);
        
        let addr = table.register(&ident, symbol);
        
        assert_eq!(table.get_mapping(ident.id()), addr);
        assert_eq!(table.get_mapping_opt(ident.id()), Some(addr));
        
        let non_existent_id = NodeId(12341);
        assert_eq!(table.get_mapping_opt(non_existent_id), None);
        assert!(!table.contains_node(non_existent_id));
    }

    #[test]
    fn test_global_table_to_runtime_conversion() {
        let mut table = GlobalTable::new();
        let mut type_table = TypeTable::new();
        
        let ident = Node::dummy(Ident { span: Span::default(), value: "global_int".to_owned() });
        let int_info = type_table.register(&Type::int());
        let symbol = create_global_symbol(ident.id(), int_info);
        
        let addr = table.register(&ident, symbol);
        
        let runtime_type_table = type_table.into_conversion().unwrap();
        let runtime_table = table.into_runtime(&runtime_type_table).unwrap();
        
        assert_eq!(runtime_table.pool.len(), 1);
        assert!(runtime_table.contains_node(ident.id()));
        assert_eq!(runtime_table.get_mapping(ident.id()), addr);
        
        let runtime_global = runtime_table.get(addr);
        assert_eq!(runtime_global.type_index.0, 1);
    }

    #[test]
    fn test_global_table_runtime_conversion_with_unknown_type() {
        let mut table = GlobalTable::new();
        let mut type_table = TypeTable::new();
        
        let ident = Node::dummy(Ident { span: Span::default(), value: "unknown_global".to_owned() });
        let unknown_info = type_table.register(&Type::unknown());
        let symbol = create_global_symbol(ident.id(), unknown_info);
        
        table.register(&ident, symbol);
        
        let runtime_type_table = type_table.into_conversion().unwrap();
        let result = table.into_runtime(&runtime_type_table);
        
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_global_table_integration_with_mixed_types() {
        let mut table = GlobalTable::new();
        let mut type_table = TypeTable::new();
        
        let valid_globals = vec![
            ("counter", Type::int()),
            ("pi", Type::float()),
            ("enabled", Type::bool()),
        ];
        
        let mut node_ids = Vec::new();
        for (name, ty) in &valid_globals {
            let ident = Node::dummy(Ident { span: Span::default(), value: name.to_string() });
            let type_info = type_table.register(ty);
            let symbol = create_global_symbol(ident.id(), type_info);
            
            table.register(&ident, symbol);
            node_ids.push(ident.id());
        }
        
        let unknown_ident = Node::dummy(Ident { span: Span::default(), value: "unknown".to_owned() });
        let unknown_info = type_table.register(&Type::unknown());
        let unknown_symbol = create_global_symbol(unknown_ident.id(), unknown_info);
        table.register(&unknown_ident, unknown_symbol);
        
        let runtime_type_table = type_table.into_conversion().unwrap();
        let result = table.into_runtime(&runtime_type_table);
        
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
    }
}
