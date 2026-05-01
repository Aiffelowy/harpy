use crate::{
    analyzer::{
        analyzer::{Analyzer, Fallback},
        tables::symbol_table::SymbolId,
    },
    lexer::span::Span,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalId(pub usize);

impl GlobalId {
    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }
}
impl Fallback<GlobalId> for Analyzer {
    fn fallback(&self) -> GlobalId {
        GlobalId(0)
    }
}

#[derive(Debug, Clone)]
pub struct GlobalDef {
    pub id: Option<GlobalId>,
    pub name: String,
    pub symbol: SymbolId,
    pub span: Span,
}

#[derive(Debug)]
pub struct GlobalTable {
    pub globals: Vec<GlobalDef>,
}

impl Default for GlobalTable {
    fn default() -> Self {
        let mut s = Self {
            globals: Vec::new(),
        };
        let dummy = GlobalDef {
            id: None,
            name: "<unknown_global>".to_owned(),
            symbol: SymbolId(0),
            span: Span::default(),
        };

        s.register(dummy);

        s
    }
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
