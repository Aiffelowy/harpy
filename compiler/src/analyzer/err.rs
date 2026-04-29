use crate::lexer::span::Span;

#[derive(Debug)]
pub enum SymbolDeclError {
    AlreadyExists { name: String, original_def: Span },
    UnknownType(String),
    UnknownFunction(String),
    ArraySizeInt,
}

#[derive(Debug)]
pub enum AnalyzerError {
    SymbolDecl(SymbolDeclError),
}

impl From<SymbolDeclError> for AnalyzerError {
    fn from(value: SymbolDeclError) -> Self {
        Self::SymbolDecl(value)
    }
}
