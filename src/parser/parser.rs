use crate::{
    aliases::Result,
    err::HarpyError,
    lexer::{
        tokens::{TokenType, Tokenize},
        Lexer,
    },
    tt,
};

use super::{program::Program, Parse};

#[derive(Debug)]
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

    pub(in crate::parser) fn peek(&mut self) -> Result<&TokenType> {
        self.lexer.peek()
    }

    pub(in crate::parser) fn consume<Tok: Tokenize>(&mut self) -> Result<Tok> {
        Tok::tokenize(&mut self.lexer)
    }

    pub(in crate::parser) fn discard_next(&mut self) -> Result<()> {
        self.lexer.next_token()?;
        Ok(())
    }

    pub(in crate::parser) fn parse<P: Parse>(&mut self) -> Result<P> {
        P::parse(self)
    }

    pub(in crate::parser) fn try_parse<T: Parse>(&mut self) -> Option<T> {
        let old = self.lexer.clone();
        if let Ok(parsed) = T::parse(self) {
            return Some(parsed);
        }

        self.lexer = old;
        return None;
    }

    pub(in crate::parser) fn fork(&self) -> Self {
        Self {
            lexer: self.lexer.clone(),
            errors: vec![],
        }
    }

    pub(in crate::parser) fn unexpected<P: Parse>(&mut self, expected: &'static str) -> Result<P> {
        let t = self.lexer.next_token()?;
        let span = t.span();
        return HarpyError::lexer(
            crate::lexer::err::LexerError::UnexpectedToken(expected, t),
            span,
        );
    }

    pub(in crate::parser) fn report_error(
        &mut self,
        error: HarpyError,
        recovery_points: &[TokenType],
    ) -> Result<()> {
        self.errors.push(error);

        loop {
            let t = self.lexer.peek()?;

            if recovery_points.contains(&t) {
                break;
            }

            if matches!(t, tt!(eof) | tt!("}") | tt!(;)) {
                self.discard_next()?;
                break;
            }
            self.discard_next()?;
        }
        Ok(())
    }

    pub fn build_ast(mut self) -> std::result::Result<Program, Vec<HarpyError>> {
        match self.parse::<Program>() {
            Ok(p) => {
                if self.errors.is_empty() {
                    return Ok(p);
                }

                return Err(self.errors);
            }

            Err(e) => {
                self.errors.push(e);
                return Err(self.errors);
            }
        }
    }
}
