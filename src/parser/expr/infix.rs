use std::fmt::Display;

use super::binding_power::Bp;
use crate::lexer::span::Span;
use crate::parser::parser::Parser;
use crate::parser::Parse;
use crate::tt;

#[derive(Debug, Clone, PartialEq)]
pub enum InfixOpKind {
    Plus,
    Minus,
    Mult,
    Div,
    And,
    Or,
    Gt,
    Lt,
    Eq,
    GtEq,
    LtEq,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InfixOp {
    pub op: InfixOpKind,
    span: Span,
}

impl InfixOp {
    pub fn bp(&self) -> Bp {
        match self.op {
            InfixOpKind::Mult | InfixOpKind::Div => (60, 61),
            InfixOpKind::Plus | InfixOpKind::Minus => (50, 51),
            InfixOpKind::GtEq
            | InfixOpKind::LtEq
            | InfixOpKind::Eq
            | InfixOpKind::Lt
            | InfixOpKind::Gt => (40, 41),
            InfixOpKind::And => (30, 31),
            InfixOpKind::Or => (20, 21),
        }
        .into()
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

impl Display for InfixOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use InfixOpKind::*;
        let s = match self.op {
            Plus => "add",
            Minus => "subtract",
            Mult => "multiply",
            Div => "divide",
            And => "&&",
            Or => "||",
            Gt => ">",
            Lt => "<",
            Eq => "==",
            GtEq => ">=",
            LtEq => "<=",
        };

        write!(f, "{s}")
    }
}

impl Parse for InfixOp {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        let op = match parser.peek()? {
            tt!(+) => InfixOpKind::Plus,
            tt!(-) => InfixOpKind::Minus,
            tt!(*) => InfixOpKind::Mult,
            tt!(/) => InfixOpKind::Div,
            tt!(&&) => InfixOpKind::And,
            tt!(||) => InfixOpKind::Or,
            tt!(>) => InfixOpKind::Gt,
            tt!(<) => InfixOpKind::Lt,
            tt!(==) => InfixOpKind::Eq,
            tt!(>=) => InfixOpKind::GtEq,
            tt!(<=) => InfixOpKind::LtEq,
            _ => {
                return parser.unexpected("infix operator");
            }
        };

        let t = parser.discard_next()?;

        Ok(Self { op, span: t.span() })
    }
}
