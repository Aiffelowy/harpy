use crate::parser::parser::Parser;
use crate::parser::{parse_trait::Parse, statements::Stmt};
use crate::{t, tt};

#[derive(Debug, Clone, PartialEq)]
pub struct BlockStmt {
    stmts: Vec<Stmt>,
}

impl Parse for BlockStmt {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        let mut stmts = vec![];
        parser.consume::<t!("{")>()?;

        loop {
            if let tt!("}") = parser.peek()? {
                break;
            }

            match parser.parse::<Stmt>() {
                Ok(s) => stmts.push(s),
                Err(e) => parser.report_error(e)?,
            }
        }

        parser.consume::<t!("}")>()?;
        Ok(Self { stmts })
    }
}
