use std::num::{ParseFloatError, ParseIntError};

use crate::err::HarpyError;

use super::{span::Span, tokens::Token};

#[derive(Debug)]
pub enum LexerError {
    UnknownToken(Span),
    InvalidInt(ParseIntError),
    InvalidFloat(ParseFloatError),
    UnclosedStr(Span),
    UnexpectedToken(&'static str, Token),
    IO(std::io::Error),
}

/*
impl Into<HarpyError> for LexerError {
    fn into(self) -> HarpyError {
        HarpyError::LexerError(self)
    }
}
*/

impl From<std::io::Error> for HarpyError {
    fn from(value: std::io::Error) -> Self {
        HarpyError::LexerError(LexerError::IO(value))
    }
}

impl From<LexerError> for HarpyError {
    fn from(value: LexerError) -> Self {
        HarpyError::LexerError(value)
    }
}
