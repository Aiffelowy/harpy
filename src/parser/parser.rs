use crate::{
    aliases::Result,
    err::HarpyError,
    lexer::{
        tokens::{TokenType, Tokenize},
        Lexer,
    },
};

use super::Parse;

pub struct Parser<'parser> {
    lexer: Lexer<'parser>,
    errors: Vec<HarpyError>,
}

impl<'parser> Parser<'parser> {
    pub fn new(lexer: Lexer<'parser>) -> Self {
        Self {
            lexer,
            errors: vec![],
        }
    }

    pub fn peek(&mut self) -> Result<&TokenType> {
        self.lexer.peek()
    }

    pub fn consume<Tok: Tokenize>(&mut self) -> Result<Tok> {
        Tok::tokenize(&mut self.lexer)
    }

    pub fn discard_next(&mut self) -> Result<()> {
        self.lexer.next_token()?;
        Ok(())
    }

    pub fn parse<P: Parse>(&mut self) -> Result<P> {
        P::parse(self)
    }

    pub fn try_parse<T: Parse>(&mut self) -> Option<T> {
        let old = self.lexer.clone();
        if let Ok(parsed) = T::parse(self) {
            return Some(parsed);
        }

        self.lexer = old;
        return None;
    }

    pub fn fork(&self) -> Self {
        Self {
            lexer: self.lexer.clone(),
            errors: vec![],
        }
    }

    pub fn unexpected<P: Parse>(&mut self, expected: &'static str) -> Result<P> {
        return Err(HarpyError::LexerError(
            crate::lexer::err::LexerError::UnexpectedToken(expected, self.lexer.next_token()?),
        ));
    }
}
