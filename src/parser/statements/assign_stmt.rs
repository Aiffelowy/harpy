use crate::lexer::err::LexerError;
use crate::parser::{expr::Expr, parse_trait::Parse};
use crate::{t, tt};

#[derive(Debug, Clone, PartialEq)]
pub enum AssignOp {
    Normal,
    Add,
    Sub,
    Mult,
    Div,
}

impl Parse for AssignOp {
    fn parse(token_stream: &mut crate::lexer::Lexer) -> crate::aliases::Result<Self> {
        let s = match token_stream.peek()? {
            tt!(=) => Self::Normal,
            tt!(+=) => Self::Add,
            tt!(-=) => Self::Sub,
            tt!(*=) => Self::Mult,
            tt!(/=) => Self::Div,
            _ => {
                return Err(LexerError::UnexpectedToken(
                    "assignment operator",
                    token_stream.next_token()?,
                )
                .into())
            }
        };

        token_stream.next_token()?;

        Ok(s)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignStmt {
    op: AssignOp,
    rhs: Expr,
}

impl Parse for AssignStmt {
    fn parse(token_stream: &mut crate::lexer::Lexer) -> crate::aliases::Result<Self> {
        let op = token_stream.parse::<AssignOp>()?;
        let rhs = token_stream.parse::<Expr>()?;
        token_stream.consume::<t!(;)>()?;
        Ok(Self { op, rhs })
    }
}
