use std::collections::HashMap;

use crate::{
    aliases::Result,
    analyzer::{err::SymbolDeclError, symbols::symbols::SymbolId},
    err::HarpyError,
    lexer::span::Span,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalId(pub usize);

#[derive(Debug, Clone)]
pub struct GlobalDef {
    pub name: String,
    pub id: Option<GlobalId>,
    pub symbol: SymbolId,
    pub span: Span,
}

#[derive(Debug, Default)]
pub struct GlobalTable {
    pub globals: Vec<GlobalDef>,
    pub name_to_id: HashMap<String, GlobalId>,
}

impl GlobalTable {
    pub fn register(&mut self, mut def: GlobalDef) -> Result<GlobalId> {
        if let Some(&existing_id) = self.name_to_id.get(&def.name) {
            let existing_def = &self.globals[existing_id.0];
            return HarpyError::analyzer(
                SymbolDeclError::AlreadyExists {
                    name: def.name.clone(),
                    original_def: existing_def.span,
                }
                .into(),
                def.span,
            );
        }

        let id = GlobalId(self.globals.len());
        def.id = Some(id);

        self.name_to_id.insert(def.name.clone(), id);
        self.globals.push(def);

        Ok(id)
    }
}
