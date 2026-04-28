use crate::aliases::Result;
use crate::lexer::{err::LexerError, span::Span};

#[derive(Debug)]
pub enum HarpyErrorKind {
    Lexer(LexerError),
    IO(std::io::Error),
}

#[derive(Debug)]
pub struct HarpyError {
    span: Span,
    error: HarpyErrorKind,
}

impl HarpyError {
    pub fn lexer<T>(error: LexerError, span: Span) -> Result<T> {
        Err(Box::new(Self {
            span,
            error: HarpyErrorKind::Lexer(error),
        }))
    }
}

impl From<std::io::Error> for Box<HarpyError> {
    fn from(value: std::io::Error) -> Self {
        Box::new(HarpyError {
            span: Span::default(),
            error: HarpyErrorKind::IO(value),
        })
    }
}
