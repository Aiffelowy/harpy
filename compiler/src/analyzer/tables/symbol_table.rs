use std::collections::HashMap;

use crate::{analyzer::types::types::TypeId, lexer::span::Span, parser::node::NodeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub usize);

#[derive(Debug, Clone)]
pub struct Symbol {
    pub id: Option<SymbolId>,
    pub name: String,
    pub ty: TypeId,
    pub is_mutable: bool,
    pub declared_at: Span,
}

#[derive(Debug, Default)]
pub struct SymbolTable {
    pub symbols: Vec<Symbol>,
    pub resolutions: HashMap<NodeId, SymbolId>,
}

impl SymbolTable {
    pub fn register(&mut self, mut def: Symbol) -> SymbolId {
        let id = SymbolId(self.symbols.len());
        def.id = Some(id);
        self.symbols.push(def);
        id
    }

    pub fn get(&self, id: SymbolId) -> &Symbol {
        &self.symbols[id.0]
    }

    pub fn get_symbol(&self, id: NodeId) -> Option<&Symbol> {
        let id = self.resolutions.get(&id)?;
        Some(&self.symbols[id.0])
    }

    pub fn add_resolution(&mut self, node_id: NodeId, symbol_id: SymbolId) {
        self.resolutions.insert(node_id, symbol_id);
    }
}
