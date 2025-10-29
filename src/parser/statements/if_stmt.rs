use crate::parser::{expr::Expr, parse_trait::Parse};
use crate::{t, tt};

use super::BlockStmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ElseStmt {
    Block(BlockStmt),
    If(Box<IfStmt>),
}

impl Parse for ElseStmt {
    fn parse(token_stream: &mut crate::lexer::Lexer) -> crate::aliases::Result<Self> {
        token_stream.consume::<t!(else)>()?;
        if let tt!(if) = token_stream.peek()? {
            return Ok(Self::If(Box::new(token_stream.parse::<IfStmt>()?)));
        }

        Ok(Self::Block(token_stream.parse::<BlockStmt>()?))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    expr: Expr,
    block: BlockStmt,
    else_stmt: Option<ElseStmt>,
}

impl Parse for IfStmt {
    fn parse(token_stream: &mut crate::lexer::Lexer) -> crate::aliases::Result<Self> {
        token_stream.consume::<t!(if)>()?;
        let expr = token_stream.parse::<Expr>()?;
        let block = token_stream.parse::<BlockStmt>()?;

        let else_stmt = if let tt!(else) = token_stream.peek()? {
            Some(token_stream.parse::<ElseStmt>()?)
        } else {
            None
        };

        Ok(Self {
            expr,
            block,
            else_stmt,
        })
    }
}
