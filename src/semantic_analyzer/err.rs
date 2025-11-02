use std::fmt::Display;

use crate::{
    lexer::tokens::Ident,
    parser::{
        expr::{infix::InfixOp, prefix::PrefixOp},
        types::Type,
    },
};

#[derive(Debug)]
pub enum SemanticError {
    DuplicateSymbol(Ident),
    MissingSymbol(Ident),
    NotAFunc(Ident),
    ArgCountMismatch(Ident, usize, usize),
    ArgTypeMismatch(Type, Type),
    PrefixTypeMismatch(PrefixOp, Type),
    InfixTypeMismatch(InfixOp, Type, Type),
}

impl Display for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            _ => "sus",
        };

        write!(f, "{s}")
    }
}
