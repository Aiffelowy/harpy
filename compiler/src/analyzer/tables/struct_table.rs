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

    pub fn get<'a>(&'a self, analyzer: &'a Analyzer) -> &'a StructLayout {
        analyzer.db.struct_table.get(*self)
    }

    pub fn get_mut<'a>(&'a self, analyzer: &'a mut Analyzer) -> &'a mut StructLayout {
        analyzer.db.struct_table.get_mut(*self)
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

use std::fmt::Display;

impl Display for StructTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.layouts.is_empty() {
            return writeln!(f, "  <No structs defined>");
        }

        writeln!(f, "  ID         | Name            | Fields")?;
        writeln!(
            f,
            "  -----------+-----------------+-----------------------------------------"
        )?;

        for layout in &self.layouts {
            let id_str = match &layout.id {
                Some(id) => format!("{:?}", id),
                None => "[Unset]".to_string(),
            };

            let fields_str = if layout.fields.is_empty() {
                "{}".to_string()
            } else {
                let fields_list: Vec<String> = layout
                    .fields
                    .iter()
                    .map(|field| format!("{}: {:?}", field.name, field.ty))
                    .collect();
                format!("{{ {} }}", fields_list.join(", "))
            };

            writeln!(f, "  {:<10} | {:<15} | {}", id_str, layout.name, fields_str)?;
        }

        Ok(())
    }
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
