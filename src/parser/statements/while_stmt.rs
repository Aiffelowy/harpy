use crate::parser::{expr::Expr, parse_trait::Parse};
use crate::t;

use super::BlockStmt;

#[derive(Debug, Clone, PartialEq)]
pub struct WhileStmt {
    expr: Expr,
    block: BlockStmt,
}

impl Parse for WhileStmt {
    fn parse(token_stream: &mut crate::lexer::Lexer) -> crate::aliases::Result<Self> {
        token_stream.consume::<t!(while)>()?;
        let expr = token_stream.parse::<Expr>()?;
        let block = token_stream.parse::<BlockStmt>()?;

        Ok(Self { expr, block })
    }
}
