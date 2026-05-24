use std::num::{ParseFloatError, ParseIntError};

use crate::aliases::Result;
use crate::analyzer::tables::struct_table::StructId;
use crate::analyzer::types::types::TypeId;
use crate::lexer::span::Span;
use crate::lexer::tokens::Token;

#[derive(Debug)]
pub enum Kind {
    // LEXER ERRORS
    UnknownToken,
    InvalidInt(ParseIntError),
    InvalidFloat(ParseFloatError),
    UnclosedStr,
    UnexpectedToken { expected: &'static str, got: Token },
    UnexpectedEof,
    //PARSER ERRORS
    MultipleDefaultInSwitch,
    //SYMBOL DECL ERRORS
    AlreadyExists { name: String, original_def: Span },
    UnknownType(String),
    UnknownFunction(String),
    UnknownSymbol(String),
    ArraySizeInt,
    MissingMain,
    GlobalInFn,
    //TYPE ERRORS
    ExprNotIter,
    NotCompatible { expected: TypeId, got: TypeId },
    IfNotCompatible { expected: TypeId, got: TypeId },
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

    IO(std::io::Error),
    Custom(&'static str),
}

#[derive(Debug)]
pub struct HarpyError {
    pub(super) span: Option<Span>,
    pub(super) kind: Box<Kind>,
}

impl HarpyError {
    pub fn new(span: Span, error: Kind) -> Self {
        Self {
            kind: Box::new(error),
            span: Some(span),
        }
    }

    pub fn err<T>(span: Span, error: Kind) -> Result<T> {
        Err(Self {
            kind: Box::new(error),
            span: Some(span),
        })
    }

    pub fn no_span(kind: Kind) -> Self {
        Self {
            kind: Box::new(kind),
            span: None,
        }
    }
}

impl From<std::io::Error> for HarpyError {
    fn from(value: std::io::Error) -> Self {
        HarpyError {
            span: None,
            kind: Box::new(Kind::IO(value)),
        }
    }
}
