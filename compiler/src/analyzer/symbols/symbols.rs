use crate::{analyzer::types::types::ResolvedType, lexer::span::Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub usize);

#[derive(Debug, Clone)]
pub struct Symbol {
    id: SymbolId,
    pub name: String,
    pub ty: ResolvedType,
    pub is_mutable: bool,
    pub declared_at: Span,
}
