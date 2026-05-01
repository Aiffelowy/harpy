use crate::{analyzer::tables::symbol_table::SymbolId, lexer::span::Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalId(pub usize);

#[derive(Debug, Clone)]
pub struct GlobalDef {
    pub id: Option<GlobalId>,
    pub name: String,
    pub symbol: SymbolId,
    pub span: Span,
}

#[derive(Debug, Default)]
pub struct GlobalTable {
    pub globals: Vec<GlobalDef>,
}

impl GlobalTable {
    pub fn register(&mut self, mut def: GlobalDef) -> GlobalId {
        let id = GlobalId(self.globals.len());
        def.id = Some(id);
        self.globals.push(def);

        id
    }

    pub fn get(&self, id: GlobalId) -> &GlobalDef {
        &self.globals[id.0]
    }
}
