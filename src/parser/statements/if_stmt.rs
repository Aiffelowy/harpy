use crate::parser::parser::Parser;
use crate::parser::{expr::Expr, parse_trait::Parse};
use crate::{t, tt};

use super::BlockStmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ElseStmt {
    Block(BlockStmt),
    If(Box<IfStmt>),
}

impl Parse for ElseStmt {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        parser.consume::<t!(else)>()?;
        if let tt!(if) = parser.peek()? {
            return Ok(Self::If(Box::new(parser.parse::<IfStmt>()?)));
        }

        Ok(Self::Block(parser.parse::<BlockStmt>()?))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    expr: Expr,
    block: BlockStmt,
    else_stmt: Option<ElseStmt>,
}

impl Parse for IfStmt {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        parser.consume::<t!(if)>()?;
        let expr = parser.parse::<Expr>()?;
        let block = parser.parse::<BlockStmt>()?;

        let else_stmt = if let tt!(else) = parser.peek()? {
            Some(parser.parse::<ElseStmt>()?)
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
