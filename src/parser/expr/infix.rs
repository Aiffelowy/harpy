use crate::parser::parser::Parser;
use crate::parser::Parse;
use crate::tt;

use super::binding_power::Bp;

#[derive(Debug, Clone, PartialEq)]
pub enum InfixOp {
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

impl InfixOp {
    pub fn bp(&self) -> Bp {
        match self {
            Self::Mult | Self::Div => (60, 61),
            Self::Plus | Self::Minus => (50, 51),
            Self::GtEq | Self::LtEq | Self::Eq | Self::Lt | Self::Gt => (40, 41),
            Self::And => (30, 31),
            Self::Or => (20, 21),
        }
        .into()
    }
}

impl Parse for InfixOp {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        let s = match parser.peek()? {
            tt!(+) => Self::Plus,
            tt!(-) => Self::Minus,
            tt!(*) => Self::Mult,
            tt!(/) => Self::Div,
            tt!(&&) => Self::And,
            tt!(||) => Self::Or,
            tt!(>) => Self::Gt,
            tt!(<) => Self::Lt,
            tt!(==) => Self::Eq,
            tt!(>=) => Self::GtEq,
            tt!(<=) => Self::LtEq,
            _ => {
                return parser.unexpected("infix operator");
            }
        };

        parser.discard_next()?;

        Ok(s)
    }
}
