use std::{
    fmt::Display,
    num::{ParseFloatError, ParseIntError},
};

use crate::color::Color;

use super::tokens::Token;

#[derive(Debug)]
pub enum LexerError {
    UnknownToken,
    InvalidInt(ParseIntError),
    InvalidFloat(ParseFloatError),
    UnclosedStr,
    UnexpectedToken(&'static str, Token),
}

impl Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::UnknownToken => "Unknown token",
            Self::InvalidInt(_) => "Invalid integer",
            Self::InvalidFloat(_) => "Invalid float",
            Self::UnclosedStr => "Unclosed String",
            Self::UnexpectedToken(expected, got) => &format!(
                "expected {}{}{}, got {}\"{}\"{}",
                Color::Green,
                expected,
                Color::Reset,
                Color::Red,
                got.t,
                Color::Reset
            ),
        };

        write!(f, "{s}")
    }
}
