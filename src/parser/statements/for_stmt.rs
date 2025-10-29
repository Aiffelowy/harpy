use crate::lexer::tokens::Ident;
use crate::parser::{expr::Expr, parse_trait::Parse};
use crate::t;

use super::BlockStmt;

#[derive(Debug, Clone, PartialEq)]
pub struct IterExpr {
    from: Expr,
    to: Expr,
}

impl Parse for IterExpr {
    fn parse(token_stream: &mut crate::lexer::Lexer) -> crate::aliases::Result<Self> {
        let from = token_stream.parse::<Expr>()?;
        token_stream.consume::<t!(..)>()?;
        let to = token_stream.parse::<Expr>()?;

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
    fn parse(token_stream: &mut crate::lexer::Lexer) -> crate::aliases::Result<Self> {
        token_stream.consume::<t!(for)>()?;
        let var = token_stream.consume::<t!(ident)>()?;
        token_stream.consume::<t!(in)>()?;
        let iter = token_stream.parse::<IterExpr>()?;
        let block = token_stream.parse::<BlockStmt>()?;
        Ok(Self { var, iter, block })
    }
}
