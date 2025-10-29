use crate::aliases::Result;

use super::parser::Parser;

pub trait Parse
where
    Self: Sized,
{
    fn parse(parser: &mut Parser) -> Result<Self>;
}
