use crate::parser::parser::Parser;
use crate::parser::{expr::Expr, parse_trait::Parse};
use crate::t;

use super::BlockStmt;

#[derive(Debug, Clone, PartialEq)]
pub struct WhileStmt {
    expr: Expr,
    block: BlockStmt,
}

impl Parse for WhileStmt {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        parser.consume::<t!(while)>()?;
        let expr = parser.parse::<Expr>()?;
        let block = parser.parse::<BlockStmt>()?;

        Ok(Self { expr, block })
    }
}
