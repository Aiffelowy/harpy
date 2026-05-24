use std::{iter::Peekable, str::Chars};

use crate::{
    aliases::Result,
    lexer::tokens::TokenType,
    source::{source_map::FileId, SourceFile},
    tt,
};

use super::{span::Position, tokens::Token};

#[derive(Debug, Clone)]
pub struct Lexer<'lexer> {
    chars: Peekable<Chars<'lexer>>,
    position: Position,
    peeked: Option<Token>,
    file_id: FileId,
}

impl<'lexer> Lexer<'lexer> {
    pub fn new(source: &'lexer SourceFile) -> Self {
        Self {
            chars: source.text.chars().peekable(),
            position: Position::default(),
            peeked: None,
            file_id: source.id,
        }
    }

    pub fn file_id(&self) -> FileId {
        self.file_id
    }

    pub(in crate::lexer) fn next_char(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        self.position.byte += c.len_utf8();

        if c == '\n' {
            self.position.column = 1;
            self.position.line += 1;
        } else {
            self.position.column += 1;
        }

        Some(c)
    }

    pub(in crate::lexer) fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    pub fn position(&self) -> Position {
        if let Some(token) = &self.peeked {
            return token.span.start;
        }
        self.position
    }

    pub(in crate::lexer) fn skip_line_comments(&mut self) {
        while let Some(c) = self.next_char() {
            if c == '\n' {
                break;
            }
        }
    }

    pub(in crate::lexer) fn skip_multi_comments(&mut self) {
        while let Some(c) = self.next_char() {
            if c != '*' {
                continue;
            }

            if let Some('/') = self.next_char() {
                break;
            }
        }
    }

    pub(in crate::lexer) fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if !c.is_whitespace() {
                break;
            }

            self.next_char();
        }
    }

    fn get_next(&mut self) -> Result<Token> {
        self.skip_whitespace();

        let next = Token::parse(self)?;

        match next.t {
            tt!("//") => {
                self.skip_line_comments();
                self.get_next()
            }
            tt!("/*") => {
                self.skip_multi_comments();
                self.get_next()
            }
            _ => Ok(next),
        }
    }

    pub fn next_token(&mut self) -> Result<Token> {
        if let Some(token) = self.peeked.take() {
            return Ok(token);
        }

        self.get_next()
    }

    pub fn peek(&mut self) -> Result<&TokenType> {
        if self.peeked.is_none() {
            self.peeked = Some(self.get_next()?);
        }

        Ok(&self.peeked.as_ref().unwrap().t)
    }
}
