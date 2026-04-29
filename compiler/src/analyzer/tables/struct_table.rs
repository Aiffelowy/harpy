use std::{any::TypeId, collections::HashMap};

use crate::{aliases::Result, analyzer::err::SymbolDeclError, err::HarpyError, lexer::span::Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StructId(pub usize);

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: TypeId,
    pub offset: usize,
}

#[derive(Debug, Clone)]
pub struct StructLayout {
    pub name: String,
    pub id: Option<StructId>,
    pub fields: Vec<Field>,
    pub total_size: usize,
    pub span: Span,
}

#[derive(Debug, Default)]
pub struct StructTable {
    pub layouts: Vec<StructLayout>,
    pub name_to_id: HashMap<String, StructId>,
}

impl StructTable {
    pub fn register(&mut self, mut layout: StructLayout) -> Result<StructId> {
        if let Some(existing_id) = self.name_to_id.get(&layout.name) {
            let existing_layout = &self.layouts[existing_id.0];
            return HarpyError::analyzer(
                SymbolDeclError::AlreadyExists {
                    name: layout.name.clone(),
                    original_def: existing_layout.span,
                }
                .into(),
                layout.span,
            );
        }

        let id = StructId(self.layouts.len());
        layout.id = Some(id);

        self.name_to_id.insert(layout.name.clone(), id);
        self.layouts.push(layout);

        Ok(id)
    }
}
