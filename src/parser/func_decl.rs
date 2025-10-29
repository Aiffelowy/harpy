use super::{statements::BlockStmt, types::Type, Parse};
use crate::{
    aliases::Result,
    lexer::{tokens::Ident, Lexer},
    t, tt,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    name: Ident,
    ttype: Type,
}

impl Parse for Param {
    fn parse(token_stream: &mut crate::lexer::Lexer) -> crate::aliases::Result<Self> {
        let name = token_stream.consume::<t!(ident)>()?;
        token_stream.consume::<t!(:)>()?;
        let ttype = token_stream.parse::<Type>()?;
        Ok(Self { name, ttype })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FuncDelc {
    name: Ident,
    params: Vec<Param>,
    return_type: Option<Type>,
    block: BlockStmt,
}

impl FuncDelc {
    fn parse_params(token_stream: &mut Lexer, params: &mut Vec<Param>) -> Result<()> {
        let first = token_stream.parse::<Param>()?;
        params.push(first);
        loop {
            if let tt!(,) = token_stream.peek()? {
                token_stream.consume::<t!(,)>()?;
                params.push(token_stream.parse::<Param>()?);
            } else {
                break;
            }
        }

        Ok(())
    }
}

impl Parse for FuncDelc {
    fn parse(token_stream: &mut Lexer) -> Result<Self> {
        token_stream.consume::<t!(fn)>()?;
        let name = token_stream.consume::<t!(ident)>()?;
        token_stream.consume::<t!("(")>()?;
        let mut params = vec![];

        if let tt!(ident) = token_stream.peek()? {
            Self::parse_params(token_stream, &mut params)?;
        }

        token_stream.consume::<t!(")")>()?;

        let mut return_type = None;

        if let tt!(->) = token_stream.peek()? {
            token_stream.consume::<t!(->)>()?;
            return_type = Some(token_stream.parse::<Type>()?);
        }

        let block = token_stream.parse::<BlockStmt>()?;

        Ok(Self {
            name,
            params,
            return_type,
            block,
        })
    }
}
