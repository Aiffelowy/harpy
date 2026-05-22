use std::fmt::Display;
use std::num::{ParseFloatError, ParseIntError};

use crate::aliases::Result;
use crate::analyzer::tables::struct_table::StructId;
use crate::analyzer::types::types::TypeId;
use crate::color::Color;
use crate::lexer::span::Span;
use crate::lexer::tokens::Token;
use crate::source::SourceFile;

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

impl Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct HarpyError {
    span: Option<Span>,
    error: Box<Kind>,
}

impl HarpyError {
    pub fn new(span: Span, error: Kind) -> Self {
        Self {
            error: Box::new(error),
            span: Some(span),
        }
    }

    pub fn err<T>(span: Span, error: Kind) -> Result<T> {
        Err(Self {
            error: Box::new(error),
            span: Some(span),
        })
    }
}

impl From<std::io::Error> for HarpyError {
    fn from(value: std::io::Error) -> Self {
        HarpyError {
            span: None,
            error: Box::new(Kind::IO(value)),
        }
    }
}

impl HarpyError {
    pub fn print_diagnostic(&self, source: &SourceFile, file_name: &str) {
        let line_num = self.span.unwrap_or_default().start.line;
        let col_num = self.span.unwrap_or_default().start.column;
        let msg = self.error.to_string();

        let line_text = source
            .get_line(line_num.saturating_sub(1))
            .unwrap_or("")
            .trim_end();

        let span_len = self
            .span
            .unwrap_or_default()
            .end
            .byte
            .saturating_sub(self.span.unwrap_or_default().start.byte);
        let squiggle_len = std::cmp::max(1, span_len);

        let safe_squiggle_len = std::cmp::min(
            squiggle_len,
            line_text.len().saturating_sub(col_num.saturating_sub(1)),
        );

        let line_num_str = line_num.to_string();
        let margin = " ".repeat(line_num_str.len());
        let indent = " ".repeat(col_num.saturating_sub(1));
        let squiggles = "^".repeat(safe_squiggle_len);

        println!(
            "{}{}Error: {}{}{}",
            Color::Bold,
            Color::Red,
            Color::Reset,
            Color::Bold,
            msg
        );

        println!(
            "{}  --> {}:{}:{}{}",
            Color::Cyan,
            file_name,
            line_num,
            col_num,
            Color::Reset
        );

        println!("{} {} |{}", Color::Cyan, margin, Color::Reset);

        println!(
            "{} {} |{} {}",
            Color::Cyan,
            line_num_str,
            Color::Reset,
            line_text
        );

        println!(
            "{} {} |{} {}{}{}{}\n",
            Color::Cyan,
            margin,
            Color::Reset,
            indent,
            Color::Red,
            squiggles,
            Color::Reset
        );
    }
}
