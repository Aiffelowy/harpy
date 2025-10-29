use crate::lexer::err::LexerError;

#[derive(Debug)]
pub enum HarpyError {
    LexerError(LexerError),
    ParserError(Vec<HarpyError>),
}
