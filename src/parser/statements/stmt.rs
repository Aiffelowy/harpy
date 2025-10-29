use crate::{
    parser::{expr::Expr, parser::Parser, Parse},
    t, tt,
};

use super::{
    assign_stmt::AssignOp, BlockStmt, ForStmt, IfStmt, LetStmt, LoopStmt, ReturnStmt, WhileStmt,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    LetStmt(LetStmt),
    IfStmt(IfStmt),
    ForStmt(ForStmt),
    WhileStmt(WhileStmt),
    LoopStmt(LoopStmt),
    ReturnStmt(ReturnStmt),
    AssignStmt(Expr, AssignOp, Expr),
    BlockStmt(BlockStmt),
    Expr(Expr),
}

impl Parse for Stmt {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        let s = match parser.peek()? {
            tt!("{") => Self::BlockStmt(BlockStmt::parse(parser)?),
            tt!(let) => Self::LetStmt(LetStmt::parse(parser)?),
            tt!(if) => Self::IfStmt(IfStmt::parse(parser)?),
            tt!(for) => Self::ForStmt(ForStmt::parse(parser)?),
            tt!(while) => Self::WhileStmt(WhileStmt::parse(parser)?),
            tt!(loop) => Self::LoopStmt(LoopStmt::parse(parser)?),
            tt!(return) => Self::ReturnStmt(ReturnStmt::parse(parser)?),
            _ => {
                let expr = parser.parse::<Expr>()?;
                if let Some(assign) = parser.try_parse::<AssignOp>() {
                    Self::AssignStmt(expr, assign, parser.parse::<Expr>()?)
                } else {
                    parser.consume::<t!(;)>()?;
                    Self::Expr(expr)
                }
            }
        };

        Ok(s)
    }
}
