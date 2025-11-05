use std::fmt::Display;

use crate::lexer::span::Span;
use crate::parser::parser::Parser;
use crate::parser::Parse;
use crate::tt;

use super::binding_power::Bp;

#[derive(Debug, Clone, PartialEq)]
pub enum PrefixOpKind {
    Minus,
    Plus,
    Neg,
    Star,
    Box,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrefixOp {
    span: Span,
    pub op: PrefixOpKind,
}

impl PrefixOp {
    pub fn bp(&self) -> Bp {
        match self.op {
            PrefixOpKind::Box => (0, 19),
            _ => (0, 70),
        }
        .into()
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

impl Display for PrefixOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use PrefixOpKind::*;
        let s = match self.op {
            Minus => "-",
            Plus => "+",
            Neg => "-",
            Star => "*",
            Box => "box ",
        };

        write!(f, "{s}")
    }
}

impl Parse for PrefixOp {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        let op = match parser.peek()? {
            tt!(+) => PrefixOpKind::Plus,
            tt!(-) => PrefixOpKind::Minus,
            tt!(!) => PrefixOpKind::Neg,
            tt!(*) => PrefixOpKind::Star,
            tt!(box) => PrefixOpKind::Box,
            _ => {
                return parser.unexpected("prefix operator");
            }
        };

        let t = parser.discard_next()?;

        Ok(Self { op, span: t.span() })
    }
}
