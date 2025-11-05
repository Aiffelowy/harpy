use std::fmt::Display;

use crate::{
    aliases::TypeInfoRc,
    color::Color,
    lexer::tokens::Ident,
    parser::{
        expr::{infix::InfixOp, prefix::PrefixOp, Expr},
        node::Node,
        types::{Type, TypeSpanned},
    },
};

use super::symbol_info::SymbolInfoKind;

#[derive(Debug)]
pub enum SemanticError {
    DuplicateSymbol(Node<Ident>),
    MissingSymbol(Ident),
    NotAFunc(Ident),
    ArgCountMismatch(Ident, usize, usize),
    ArgTypeMismatch(Type, TypeInfoRc),
    PrefixTypeMismatch(PrefixOp, Type),
    InfixTypeMismatch(InfixOp, Type, Type),
    LetTypeMismatch(TypeSpanned, TypeInfoRc),
    ForTypeMismatch(TypeInfoRc, TypeInfoRc),
    WhileTypeMismatch(TypeInfoRc),
    IfTypeMismatch(TypeInfoRc),
    ReturnNotInFunc,
    ReturnTypeMismatch(TypeInfoRc, TypeInfoRc),
    AssignTypeMismatch(TypeInfoRc, TypeInfoRc),
    AssignToConst(Node<Expr>),
    CreatedMutableBorrowWhileImmutableBorrow,
    AlreadyMutablyBorrowed,
    InvalidBorrow,
    InvalidVarBorrow(SymbolInfoKind),
    BorrowMutNonMutable,
    MissingMain,
    UnresolvedType,
    PointerToRef,
    LifetimeMismatch,
    ReturnRefToLocal,
}

impl Display for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use SemanticError::*;

        let s = match self {
            DuplicateSymbol(i) => format!(
                "redefinition of symbol {}{}{}",
                Color::Red,
                i.value(),
                Color::Reset
            ),
            MissingSymbol(i) => format!(
                "use of undeclared symbol {}{}{}",
                Color::Red,
                i.value(),
                Color::Reset
            ),
            NotAFunc(i) => format!("\"{}\" is not a function", i.value()),

            ArgCountMismatch(_i, got, expected) => format!(
                "expected {}{}{} arguments, got {}{}{}",
                Color::Green,
                expected,
                Color::Reset,
                Color::Red,
                got,
                Color::Reset,
            ),

            ArgTypeMismatch(got, expected) => format!(
                "incorrect arguments; expected {}{}{} got {}{}{}",
                Color::Green,
                expected.ttype,
                Color::Reset,
                Color::Red,
                got,
                Color::Reset,
            ),

            PrefixTypeMismatch(op, ty) => {
                format!(
                    "cannot apply {}{}{} to {}{}{}",
                    Color::Green,
                    op,
                    Color::Reset,
                    Color::Red,
                    ty,
                    Color::Reset
                )
            }

            InfixTypeMismatch(op, lhs, rhs) => format!(
                "cannot {}{} {}{}{} to {}{}{}",
                Color::Green,
                op,
                Color::Red,
                lhs,
                Color::Reset,
                Color::Red,
                rhs,
                Color::Reset
            ),

            LetTypeMismatch(i, ty) => format!(
                "cannot assign {}{}{} to {}{}{}",
                Color::Red,
                ty,
                Color::Reset,
                Color::Green,
                i,
                Color::Reset
            ),

            ForTypeMismatch(got, expected) => format!(
                "type mismatch, expected {}{}{} got {}{}{}",
                Color::Green,
                expected,
                Color::Reset,
                Color::Red,
                got,
                Color::Reset,
            ),

            WhileTypeMismatch(got) => format!(
                "type mismatch, expected {}bool{} got {}{}{}",
                Color::Green,
                Color::Reset,
                Color::Red,
                got,
                Color::Reset,
            ),

            IfTypeMismatch(got) => format!(
                "type mismatch, expected {}bool{} got {}{}{}",
                Color::Green,
                Color::Reset,
                Color::Red,
                got,
                Color::Reset,
            ),

            ReturnNotInFunc => format!(
                "{}return{} used outside a {}function{}",
                Color::Red,
                Color::Reset,
                Color::Green,
                Color::Reset
            ),

            ReturnTypeMismatch(got, expected) => format!(
                "expected {}{}{} because of return type, got {}{}{}",
                Color::Green,
                expected,
                Color::Reset,
                Color::Red,
                got,
                Color::Reset,
            ),

            AssignTypeMismatch(got, expected) => format!(
                "cannot assign {}{}{} to {}{}{}",
                Color::Red,
                got,
                Color::Reset,
                Color::Green,
                expected,
                Color::Reset,
            ),

            AssignToConst(expr) => format!(
                "{}{}{} is not {}mutable{}",
                Color::Red,
                expr,
                Color::Reset,
                Color::Green,
                Color::Reset
            ),
            MissingMain => format!("missing {}main{}", Color::Red, Color::Reset),
            UnresolvedType => format!("internal error: unresolved type"),
            PointerToRef => format!("cannot create a pointer to a reference"),
            CreatedMutableBorrowWhileImmutableBorrow => {
                format!("cannot borrow as mutable; already borrowed as immutable")
            }
            AlreadyMutablyBorrowed => format!("cannot borrow; already borrowed as mutable"),
            InvalidBorrow => format!("cannot borrow this value"),
            BorrowMutNonMutable => format!("cannot borrow immutable value as mutable"),
            LifetimeMismatch => format!("borrow outlives base variable"),
            InvalidVarBorrow(k) => format!("cannot borrow {k}s"),
            ReturnRefToLocal => format!("cannot return a reference to a local variable"),
        };

        write!(f, "{s}")
    }
}
