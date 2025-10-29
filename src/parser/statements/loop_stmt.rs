use crate::parser::parse_trait::Parse;
use crate::t;

use super::BlockStmt;

#[derive(Debug, Clone, PartialEq)]
pub struct LoopStmt {
    block: BlockStmt,
}

impl Parse for LoopStmt {
    fn parse(token_stream: &mut crate::lexer::Lexer) -> crate::aliases::Result<Self> {
        token_stream.consume::<t!(loop)>()?;
        let block = token_stream.parse::<BlockStmt>()?;
        Ok(Self { block })
    }
}
