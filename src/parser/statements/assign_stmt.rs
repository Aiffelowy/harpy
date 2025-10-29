use crate::parser::parser::Parser;
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
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        let s = match parser.peek()? {
            tt!(=) => Self::Normal,
            tt!(+=) => Self::Add,
            tt!(-=) => Self::Sub,
            tt!(*=) => Self::Mult,
            tt!(/=) => Self::Div,
            _ => {
                return parser.unexpected("assignment operator");
            }
        };

        parser.discard_next()?;

        Ok(s)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignStmt {
    op: AssignOp,
    rhs: Expr,
}

impl Parse for AssignStmt {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        let op = parser.parse::<AssignOp>()?;
        let rhs = parser.parse::<Expr>()?;
        parser.consume::<t!(;)>()?;
        Ok(Self { op, rhs })
    }
}
