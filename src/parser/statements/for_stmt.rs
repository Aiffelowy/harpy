use crate::lexer::tokens::Ident;
use crate::parser::parser::Parser;
use crate::parser::{expr::Expr, parse_trait::Parse};
use crate::t;

use super::BlockStmt;

#[derive(Debug, Clone, PartialEq)]
pub struct IterExpr {
    from: Expr,
    to: Expr,
}

impl Parse for IterExpr {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        let from = parser.parse::<Expr>()?;
        parser.consume::<t!(..)>()?;
        let to = parser.parse::<Expr>()?;

        Ok(Self { from, to })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForStmt {
    var: Ident,
    iter: IterExpr,
    block: BlockStmt,
}

impl Parse for ForStmt {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        parser.consume::<t!(for)>()?;
        let var = parser.consume::<t!(ident)>()?;
        parser.consume::<t!(in)>()?;
        let iter = parser.parse::<IterExpr>()?;
        let block = parser.parse::<BlockStmt>()?;
        Ok(Self { var, iter, block })
    }
}
