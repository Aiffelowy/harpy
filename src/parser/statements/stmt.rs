use crate::{
    parser::{expr::Expr, Parse},
    t, tt,
};

use super::{AssignStmt, BlockStmt, ForStmt, IfStmt, LetStmt, LoopStmt, ReturnStmt, WhileStmt};

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    LetStmt(LetStmt),
    IfStmt(IfStmt),
    ForStmt(ForStmt),
    WhileStmt(WhileStmt),
    LoopStmt(LoopStmt),
    ReturnStmt(ReturnStmt),
    AssignStmt(Expr, AssignStmt),
    BlockStmt(BlockStmt),
    Expr(Expr),
}

impl Parse for Stmt {
    fn parse(token_stream: &mut crate::lexer::Lexer) -> crate::aliases::Result<Self> {
        let s = match token_stream.peek()? {
            tt!("{") => Self::BlockStmt(BlockStmt::parse(token_stream)?),
            tt!(let) => Self::LetStmt(LetStmt::parse(token_stream)?),
            tt!(if) => Self::IfStmt(IfStmt::parse(token_stream)?),
            tt!(for) => Self::ForStmt(ForStmt::parse(token_stream)?),
            tt!(while) => Self::WhileStmt(WhileStmt::parse(token_stream)?),
            tt!(loop) => Self::LoopStmt(LoopStmt::parse(token_stream)?),
            tt!(return) => Self::ReturnStmt(ReturnStmt::parse(token_stream)?),
            _ => {
                let expr = token_stream.parse::<Expr>()?;
                if let Some(assign) = token_stream.try_parse::<AssignStmt>() {
                    Self::AssignStmt(expr, assign)
                } else {
                    token_stream.consume::<t!(;)>()?;
                    Self::Expr(expr)
                }
            }
        };

        Ok(s)
    }
}
