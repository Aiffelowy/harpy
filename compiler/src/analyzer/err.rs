use crate::lexer::span::Span;

#[derive(Debug)]
pub enum SymbolDeclError {
    AlreadyExists { name: String, original_def: Span },
    UnknownType(String),
    UnknownFunction(String),
    UnknownSymbol(String),
    ArraySizeInt,
    MissingMain,
}

#[derive(Debug)]
pub enum SymbolResError {
    GlobalInFn,
}

#[derive(Debug)]
pub enum AnalyzerError {
    SymbolDecl(SymbolDeclError),
    SymbolRes(SymbolResError),
}

impl From<SymbolDeclError> for AnalyzerError {
    fn from(value: SymbolDeclError) -> Self {
        Self::SymbolDecl(value)
    }
}

impl From<SymbolResError> for AnalyzerError {
    fn from(value: SymbolResError) -> Self {
        Self::SymbolRes(value)
    }
}
