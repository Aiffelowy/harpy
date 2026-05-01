use crate::{
    analyzer::{
        analyzer::{Analyzer, Fallback},
        types::types::TypeId,
    },
    lexer::span::Span,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StructId(pub usize);

impl StructId {
    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }
}
impl Fallback<StructId> for Analyzer {
    fn fallback(&self) -> StructId {
        StructId(0)
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: TypeId,
}

#[derive(Debug, Clone)]
pub struct StructLayout {
    pub name: String,
    pub id: Option<StructId>,
    pub fields: Vec<Field>,
    pub span: Span,
}

impl StructLayout {
    pub fn skeleton(name: String, span: Span) -> Self {
        StructLayout {
            name,
            id: None,
            fields: Vec::new(),
            span,
        }
    }

    pub fn set_fields(&mut self, fields: Vec<Field>) {
        self.fields = fields
    }
}

#[derive(Debug)]
pub struct StructTable {
    pub layouts: Vec<StructLayout>,
}

impl Default for StructTable {
    fn default() -> Self {
        let mut s = Self {
            layouts: Vec::new(),
        };

        let dummy = StructLayout::skeleton("<unknown_struct>".to_owned(), Span::default());
        s.register(dummy);
        s
    }
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

    pub fn get_mut(&mut self, id: StructId) -> &mut StructLayout {
        &mut self.layouts[id.0]
    }
}
