use crate::{analyzer::types::types::TypeId, lexer::span::Span};

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
}

impl StructTable {
    pub fn register(&mut self, mut layout: StructLayout) -> StructId {
        let id = StructId(self.layouts.len());
        layout.id = Some(id);
        self.layouts.push(layout);

        id
    }

    pub fn get(&self, id: StructId) -> &StructLayout {
        &self.layouts[id.0]
    }
}
