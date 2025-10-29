use crate::tt;
use crate::{lexer::err::LexerError, parser::Parse};

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
    fn parse(token_stream: &mut crate::lexer::Lexer) -> crate::aliases::Result<Self> {
        let s = match token_stream.peek()? {
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
                return Err(LexerError::UnexpectedToken(
                    "infix operator",
                    token_stream.next_token()?,
                )
                .into())
            }
        };

        token_stream.next_token()?;

        Ok(s)
    }
}
