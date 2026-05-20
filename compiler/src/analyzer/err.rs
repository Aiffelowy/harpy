use crate::{
    analyzer::{tables::struct_table::StructId, types::types::TypeId},
    lexer::span::Span,
};

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
pub enum TypeCheckError {
    ExprNotIter,
    NotCompatible(TypeId, TypeId),
    InvalidLValue,
    InvalidArithmetic(TypeId, TypeId),
    InvalidRelational(TypeId, TypeId),
    InvalidPrefix(TypeId),
    UnknownField(StructId, String),
    NotAStruct(TypeId),
    NotAnArray(TypeId),
    MissingField(StructId, String),
    ArgumentCountMismatch(usize, usize),
    NotCallable(TypeId),
    ReturnOutsideFn,
    ContinueOutsideLoop,
    BreakOutsideLoop,
    MissingDefaultBranch,
    TypeAnnotationsNeeded,
    UnexpectedDot,
    MissingLoopStart,
    RecursiveBox,
    RecursiveRef,
    BoxedRef,
    RefInStruct,
}

#[derive(Debug)]
pub enum AnalyzerError {
    SymbolDecl(SymbolDeclError),
    SymbolRes(SymbolResError),
    TypeCheck(TypeCheckError),
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

impl From<TypeCheckError> for AnalyzerError {
    fn from(value: TypeCheckError) -> Self {
        Self::TypeCheck(value)
    }
}
