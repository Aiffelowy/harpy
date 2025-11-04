use crate::{
    aliases::Result,
    err::HarpyError,
    lexer::{
        span::Span,
        tokens::{Token, TokenType, Tokenize},
        Lexer,
    },
    tt,
};

use super::{
    node::{Node, NodeId},
    program::Program,
    Parse,
};

#[derive(Debug)]
pub struct Parser<'parser> {
    lexer: Lexer<'parser>,
    errors: Vec<HarpyError>,
    next_id: NodeId,
}

impl<'parser> Parser<'parser> {
    pub fn new(lexer: Lexer<'parser>) -> Self {
        Self {
            lexer,
            errors: vec![],
            next_id: NodeId(0),
        }
    }

    fn next_id(&mut self) -> NodeId {
        let i = self.next_id;
        self.next_id.0 += 1;
        i
    }

    pub(in crate::parser) fn peek(&mut self) -> Result<&TokenType> {
        self.lexer.peek()
    }

    pub(in crate::parser) fn consume<Tok: Tokenize>(&mut self) -> Result<Tok> {
        Tok::tokenize(&mut self.lexer)
    }

    pub(in crate::parser) fn discard_next(&mut self) -> Result<Token> {
        self.lexer.next_token()
    }

    pub(in crate::parser) fn parse_node<P: Parse>(&mut self) -> Result<Node<P>> {
        let start = self.lexer.current_position();
        let value = P::parse(self)?;
        let end = self.lexer.current_position();

        Ok(Node::<P>::new(self.next_id(), Span::new(start, end), value))
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
            next_id: self.next_id,
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
