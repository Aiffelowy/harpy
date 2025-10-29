use crate::tt;
use crate::{lexer::err::LexerError, parser::Parse};

use super::binding_power::Bp;

#[derive(Debug, Clone, PartialEq)]
pub enum PrefixOp {
    Minus,
    Plus,
    Neg,
    Ref,
    Star,
    Box,
}

impl PrefixOp {
    pub fn bp(&self) -> Bp {
        match self {
            Self::Box => (0, 19),
            _ => (0, 70),
        }
        .into()
    }
}

impl Parse for PrefixOp {
    fn parse(token_stream: &mut crate::lexer::Lexer) -> crate::aliases::Result<Self> {
        let s = match token_stream.peek()? {
            tt!(+) => Self::Plus,
            tt!(-) => Self::Minus,
            tt!(!) => Self::Neg,
            tt!(&) => Self::Ref,
            tt!(*) => Self::Star,
            tt!(box) => Self::Box,
            _ => {
                return Err(LexerError::UnexpectedToken(
                    "prefix operator",
                    token_stream.next_token()?,
                )
                .into())
            }
        };

        token_stream.next_token()?;

        Ok(s)
    }
}
