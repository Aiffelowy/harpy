use crate::{
    aliases::Result,
    lexer::tokens::{Ident, Literal},
};

use super::parser::Parser;

pub trait Parse
where
    Self: Sized,
{
    fn parse(parser: &mut Parser) -> Result<Self>;
}

impl Parse for Ident {
    fn parse(parser: &mut Parser) -> Result<Self> {
        parser.consume()
    }
}

impl Parse for Literal {
    fn parse(parser: &mut Parser) -> Result<Self> {
        parser.consume()
    }
}
