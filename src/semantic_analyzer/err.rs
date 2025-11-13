use std::fmt::Display;

use crate::{
    aliases::TypeInfoRc,
    color::Color,
    lexer::tokens::Ident,
    parser::{
        expr::{infix::InfixOp, prefix::PrefixOp, Expr},
        node::Node,
        types::Type,
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
    LetTypeMismatch(Type, TypeInfoRc),
    ForTypeMismatch(TypeInfoRc, Type),
    WhileTypeMismatch(TypeInfoRc),
    IfTypeMismatch(TypeInfoRc),
    ReturnNotInFunc,
    ReturnTypeMismatch(TypeInfoRc, TypeInfoRc),
    AssignTypeMismatch(TypeInfoRc, Type),
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
    AssignToRValue,
    UninitializedVar,
    CantInferType,
    NotAllPathsReturn,
}

impl Display for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Color::*;
        use SemanticError::*;

        let s = match self {
            DuplicateSymbol(i) => format!("redefinition of symbol {}{}{}", Red, i.value(), Reset),
            MissingSymbol(i) => format!("use of undeclared symbol {}{}{}", Red, i.value(), Reset),
            NotAFunc(i) => format!("\"{}\" is not a function", i.value()),

            ArgCountMismatch(_i, got, expected) => format!(
                "expected {}{}{} arguments, got {}{}{}",
                Green, expected, Reset, Red, got, Reset,
            ),

            ArgTypeMismatch(got, expected) => format!(
                "incorrect arguments; expected {}{}{} got {}{}{}",
                Green, expected.ttype, Reset, Red, got, Reset,
            ),

            PrefixTypeMismatch(op, ty) => {
                format!(
                    "cannot apply {}{}{} to {}{}{}",
                    Green, op, Reset, Red, ty, Reset
                )
            }

            InfixTypeMismatch(op, lhs, rhs) => format!(
                "cannot {}{} {}{}{} to {}{}{}",
                Green, op, Red, lhs, Reset, Red, rhs, Reset
            ),

            LetTypeMismatch(i, ty) => format!(
                "cannot assign {}{}{} to {}{}{}",
                Red, ty, Reset, Green, i, Reset
            ),

            ForTypeMismatch(got, expected) => format!(
                "type mismatch, expected {}{}{} got {}{}{}",
                Green, expected, Reset, Red, got, Reset,
            ),

            WhileTypeMismatch(got) => format!(
                "type mismatch, expected {}bool{} got {}{}{}",
                Green, Reset, Red, got, Reset,
            ),

            IfTypeMismatch(got) => format!(
                "type mismatch, expected {}bool{} got {}{}{}",
                Green, Reset, Red, got, Reset,
            ),

            ReturnNotInFunc => format!(
                "{}return{} used outside a {}function{}",
                Red, Reset, Green, Reset
            ),

            ReturnTypeMismatch(got, expected) => format!(
                "expected {}{}{} because of return type, got {}{}{}",
                Green, expected, Reset, Red, got, Reset,
            ),

            AssignTypeMismatch(got, expected) => format!(
                "cannot assign {}{}{} to {}{}{}",
                Red, got, Reset, Green, expected, Reset,
            ),

            AssignToConst(expr) => {
                format!("{}{}{} is not {}mutable{}", Red, expr, Reset, Green, Reset)
            }
            MissingMain => format!("missing {}main{}", Red, Reset),
            UnresolvedType => format!("internal error: unresolved type"),
            PointerToRef => {
                format!("cannot create a {Red}pointer{Reset} to a {Green}reference{Reset}")
            }
            CreatedMutableBorrowWhileImmutableBorrow => {
                format!("cannot {Green}borrow{Reset} as {Red}mutable{Reset}; already {Green}borrowed{Reset} as {Red}immutable{Reset}")
            }
            AlreadyMutablyBorrowed => format!(
                "cannot {Red}borrow{Reset}; already {Green}borrowed{Reset} as {Red}mutable{Reset}"
            ),
            InvalidBorrow => format!("cannot {Green}borrow{Reset} this {Red}value{Reset}"),
            BorrowMutNonMutable => format!(
                "cannot {Green}borrow{Reset} {Red}immutable{Reset} value as {Red}mutable{Reset}"
            ),
            LifetimeMismatch => format!("{Red}borrow{Reset} outlives {Green}base{Reset} variable"),
            InvalidVarBorrow(k) => format!("cannot {Green}borrow{Reset} {Red}{k}s{Reset}"),
            ReturnRefToLocal => format!("cannot {Red}return{Reset} a {Green}reference{Reset} to a {Red}local{Reset} variable"),
            AssignToRValue => format!("cannot {Green}assign{Reset} to {Red}rvalue{Reset}"),
            UninitializedVar => format!("{Green}variable{Reset} not {Red}initialized{Reset}"),
            CantInferType => format!("{Red}Cannot{Reset} {Green}infer{Reset} type; consider giving it a concrete {Green}type{Reset}"),
            NotAllPathsReturn => format!("not all code paths return a value")
        };

        write!(f, "{s}")
    }
}
