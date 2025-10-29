use crate::parser::{parse_trait::Parse, statements::Stmt};
use crate::{t, tt};

#[derive(Debug, Clone, PartialEq)]
pub struct BlockStmt {
    stmts: Vec<Stmt>,
}

impl Parse for BlockStmt {
    fn parse(token_stream: &mut crate::lexer::Lexer) -> crate::aliases::Result<Self> {
        let mut stmts = vec![];
        token_stream.consume::<t!("{")>()?;

        loop {
            if let tt!("}") = token_stream.peek()? {
                break;
            }

            stmts.push(token_stream.parse::<Stmt>()?);
        }

        token_stream.consume::<t!("}")>()?;
        Ok(Self { stmts })
    }
}
