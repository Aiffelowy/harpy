use super::{parser::Parser, statements::BlockStmt, types::Type, Parse};
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
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        let name = parser.consume::<t!(ident)>()?;
        parser.consume::<t!(:)>()?;
        let ttype = parser.parse::<Type>()?;
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
    fn parse_params(parser: &mut Parser, params: &mut Vec<Param>) -> Result<()> {
        let first = parser.parse::<Param>()?;
        params.push(first);
        loop {
            if let tt!(,) = parser.peek()? {
                parser.consume::<t!(,)>()?;
                params.push(parser.parse::<Param>()?);
            } else {
                break;
            }
        }

        Ok(())
    }
}

impl Parse for FuncDelc {
    fn parse(parser: &mut Parser) -> Result<Self> {
        parser.consume::<t!(fn)>()?;
        let name = parser.consume::<t!(ident)>()?;
        parser.consume::<t!("(")>()?;
        let mut params = vec![];

        if let tt!(ident) = parser.peek()? {
            Self::parse_params(parser, &mut params)?;
        }

        parser.consume::<t!(")")>()?;

        let mut return_type = None;

        if let tt!(->) = parser.peek()? {
            parser.consume::<t!(->)>()?;
            return_type = Some(parser.parse::<Type>()?);
        }

        let block = parser.parse::<BlockStmt>()?;

        Ok(Self {
            name,
            params,
            return_type,
            block,
        })
    }
}
