use crate::parser::parse_trait::Parse;
use crate::parser::parser::Parser;
use crate::t;

use super::BlockStmt;

#[derive(Debug, Clone, PartialEq)]
pub struct LoopStmt {
    block: BlockStmt,
}

impl Parse for LoopStmt {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        parser.consume::<t!(loop)>()?;
        let block = parser.parse::<BlockStmt>()?;
        Ok(Self { block })
    }
}
