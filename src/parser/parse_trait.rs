use crate::{aliases::Result, lexer::Lexer};

pub trait Parse
where
    Self: Sized,
{
    fn parse(token_stream: &mut Lexer) -> Result<Self>;
}
