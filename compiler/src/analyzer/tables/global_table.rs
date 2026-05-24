use std::fmt::Display;

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

    pub fn get<'a>(&'a self, analyzer: &'a Analyzer) -> &'a GlobalDef {
        analyzer.db.global_table.get(*self)
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

impl Display for GlobalTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.globals.is_empty() {
            return writeln!(f, "  <No globals defined>");
        }

        writeln!(f, "  ID         | Name            | Symbol")?;
        writeln!(
            f,
            "  -----------+-----------------+--------------+----------------"
        )?;

        for global in &self.globals {
            let id_str = match &global.id {
                Some(id) => format!("{:?}", id),
                None => "[Unset]".to_string(),
            };

            writeln!(
                f,
                "  {:<10} | {:<15} | {:?}",
                id_str, global.name, global.symbol
            )?;
        }

        Ok(())
    }
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
            span: Span::dummy(),
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
