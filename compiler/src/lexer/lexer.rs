use std::{iter::Peekable, str::Chars};

use crate::{aliases::Result, source::SourceFile, tt};

use super::{
    span::{Position, Span},
    tokens::{Token, TokenType},
};

#[derive(Debug, Clone)]
pub struct Lexer<'lexer> {
    chars: Peekable<Chars<'lexer>>,
    position: Position,
    next: Token,
    last_position: Position,
}

impl<'lexer> Lexer<'lexer> {
    pub fn new(buffer: &'lexer SourceFile) -> Result<Self> {
        let mut l = Self {
            chars: buffer.text.chars().peekable(),
            position: Position::default(),
            next: Token {
                t: crate::lexer::tokens::TokenType::Eof,
                span: Span::new(Position::default(), Position::default()),
            },
            last_position: Position::default(),
        };

        l.next_token()?;
        Ok(l)
    }

    pub(in crate::lexer) fn next_char(&mut self) -> Option<char> {
        if let Some(c) = self.chars.next() {
            self.position.byte += c.len_utf8();
            if c == '\n' {
                self.position.column = 1;
                self.position.line += 1;
            } else {
                self.position.column += 1;
            }
            return Some(c);
        }

        None
    }

    pub(in crate::lexer) fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    pub(in crate::lexer) fn position(&self) -> Position {
        self.position
    }

    //hacky, but works
    pub fn current_position_end(&self) -> Position {
        self.last_position
    }

    pub fn current_position_start(&self) -> Position {
        self.next.span().start
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

    pub fn next_token(&mut self) -> Result<Token> {
        loop {
            let next = Token::parse(self)?;
            match next.t {
                tt!("//") => {
                    self.skip_line_comments();
                    continue;
                }
                tt!("/*") => {
                    self.skip_multi_comments();
                    continue;
                }
                _ => (),
            }

            let current = std::mem::replace(&mut self.next, next);
            self.last_position = current.span().end;
            return Ok(current);
        }
    }

    pub fn peek(&self) -> Result<&TokenType> {
        Ok(&self.next.t)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::lexer::tokens::{Key, Lit, Sym, TokenType};

    macro_rules! make_lexer {
        ($name:ident, $input:literal) => {
            let source = SourceFile::new(Cursor::new($input)).unwrap();
            let mut $name = Lexer::new(&source).unwrap();
        };
    }

    #[test]
    fn test_token_peek_and_next() {
        make_lexer!(lexer, "let x = 42;");
        let peeked = lexer.peek().unwrap().clone();
        let token = lexer.next_token().unwrap();
        assert_eq!(token.t, peeked);
        let peeked = lexer.peek().unwrap().clone();
        let token = lexer.next_token().unwrap();
        assert_eq!(token.t, peeked);
        let peeked = lexer.peek().unwrap().clone();
        let token = lexer.next_token().unwrap();
        assert_eq!(token.t, peeked);
    }

    #[test]
    fn test_multiple_tokens() {
        make_lexer!(lexer, "let x = 5;\nlet y = 10;");
        let t1 = lexer.next_token().unwrap();
        let t2 = lexer.next_token().unwrap();
        let t3 = lexer.next_token().unwrap();
        assert!(t1.t != TokenType::Eof);
        assert!(t2.t != TokenType::Eof);
        assert!(t3.t != TokenType::Eof);
    }

    #[test]
    fn test_next_token_initial() {
        make_lexer!(lexer, "let x = 5;");

        let first = lexer.next_token().unwrap();
        assert_eq!(first.t, TokenType::Keyword(Key::Let));

        let second = lexer.next_token().unwrap();
        assert_eq!(second.t, TokenType::Ident("x".into()));

        let peeked = lexer.peek().unwrap();
        assert_eq!(*peeked, TokenType::Symbol(Sym::Assign));
    }

    #[test]
    fn test_basic_tokens() {
        make_lexer!(lexer, "let x = 42;");

        let t1 = lexer.next_token().unwrap();
        assert_eq!(t1.t, TokenType::Keyword(Key::Let));

        let t2 = lexer.next_token().unwrap();
        assert_eq!(t2.t, TokenType::Ident("x".to_string()));

        let t3 = lexer.next_token().unwrap();
        assert_eq!(t3.t, TokenType::Symbol(Sym::Assign));

        let t4 = lexer.next_token().unwrap();
        assert_eq!(t4.t, TokenType::Literal(Lit::LitInt(42)));

        let t5 = lexer.next_token().unwrap();
        assert_eq!(t5.t, TokenType::Symbol(Sym::Semi));

        let t6 = lexer.next_token().unwrap();
        assert_eq!(t6.t, TokenType::Eof);
    }

    #[test]
    fn test_multiline() {
        make_lexer!(lexer, "\nlet\n\n\nx\n=\n\n42\n\n\n;");

        let t1 = lexer.next_token().unwrap();
        assert_eq!(t1.t, TokenType::Keyword(Key::Let));

        let t2 = lexer.next_token().unwrap();
        assert_eq!(t2.t, TokenType::Ident("x".to_string()));

        let t3 = lexer.next_token().unwrap();
        assert_eq!(t3.t, TokenType::Symbol(Sym::Assign));

        let t4 = lexer.next_token().unwrap();
        assert_eq!(t4.t, TokenType::Literal(Lit::LitInt(42)));

        let t5 = lexer.next_token().unwrap();
        assert_eq!(t5.t, TokenType::Symbol(Sym::Semi));

        let t6 = lexer.next_token().unwrap();
        assert_eq!(t6.t, TokenType::Eof);
    }

    #[test]
    fn test_token_spans() {
        make_lexer!(lexer, "let x = 42;\nfoo");

        let t1 = lexer.next_token().unwrap();
        assert_eq!(t1.t, TokenType::Keyword(Key::Let));
        assert_eq!(t1.span.start.line, 1);
        assert_eq!(t1.span.start.column, 1);
        assert_eq!(t1.span.end.line, 1);
        assert_eq!(t1.span.end.column, 4);

        let t2 = lexer.next_token().unwrap();
        assert_eq!(t2.t, TokenType::Ident("x".to_string()));
        assert_eq!(t2.span.start.line, 1);
        assert_eq!(t2.span.start.column, 5);
        assert_eq!(t2.span.end.line, 1);
        assert_eq!(t2.span.end.column, 6);

        let t3 = lexer.next_token().unwrap();
        assert_eq!(t3.t, TokenType::Symbol(Sym::Assign));
        assert_eq!(t3.span.start.line, 1);
        assert_eq!(t3.span.start.column, 7);
        assert_eq!(t3.span.end.line, 1);
        assert_eq!(t3.span.end.column, 8);

        let t4 = lexer.next_token().unwrap();
        assert_eq!(t4.t, TokenType::Literal(Lit::LitInt(42)));
        assert_eq!(t4.span.start.line, 1);
        assert_eq!(t4.span.start.column, 9);
        assert_eq!(t4.span.end.line, 1);
        assert_eq!(t4.span.end.column, 11);

        let t5 = lexer.next_token().unwrap();
        assert_eq!(t5.t, TokenType::Symbol(Sym::Semi));
        assert_eq!(t5.span.start.line, 1);
        assert_eq!(t5.span.start.column, 11);
        assert_eq!(t5.span.end.line, 1);
        assert_eq!(t5.span.end.column, 12);

        let t6 = lexer.next_token().unwrap();
        assert_eq!(t6.t, TokenType::Ident("foo".to_string()));
        assert_eq!(t6.span.start.line, 2);
        assert_eq!(t6.span.start.column, 1);
        assert_eq!(t6.span.end.line, 2);
        assert_eq!(t6.span.end.column, 4);

        let t7 = lexer.next_token().unwrap();
        assert_eq!(t7.t, TokenType::Eof);
        assert_eq!(t7.span.start.line, 2);
        assert_eq!(t7.span.start.column, 4);
        assert_eq!(t7.span.end.line, 2);
        assert_eq!(t7.span.end.column, 4);
    }

    #[test]
    fn test_unicode_identifier() {
        make_lexer!(lexer, "变量 = 42");

        let t1 = lexer.next_token().unwrap();
        assert_eq!(t1.t, TokenType::Ident("变量".to_string()));

        let t2 = lexer.next_token().unwrap();
        assert_eq!(t2.t, TokenType::Symbol(Sym::Assign));

        let t3 = lexer.next_token().unwrap();
        assert_eq!(t3.t, TokenType::Literal(Lit::LitInt(42)));

        let t4 = lexer.next_token().unwrap();
        assert_eq!(t4.t, TokenType::Eof);
    }

    #[test]
    fn test_unicode_column_tracking() {
        make_lexer!(lexer, "\"foo😀\"");

        let t1 = lexer.next_token().unwrap();
        assert_eq!(t1.t, TokenType::Literal(Lit::LitStr("foo😀".to_string())));

        assert_eq!(t1.span.start.column, 1);
        assert_eq!(t1.span.end.column, 7);
        assert_eq!(t1.span.end.byte, 9);
    }

    #[test]
    fn test_literals() {
        make_lexer!(
            lexer,
            "5 21352135 5.0 0.2 235f 214F 0.4F \"string\" true false"
        );
        assert_eq!(
            lexer.next_token().unwrap().t,
            TokenType::Literal(Lit::LitInt(5))
        );
        assert_eq!(
            lexer.next_token().unwrap().t,
            TokenType::Literal(Lit::LitInt(21352135))
        );
        assert_eq!(
            lexer.next_token().unwrap().t,
            TokenType::Literal(Lit::LitFloat(5.0f64.to_bits()))
        );
        assert_eq!(
            lexer.next_token().unwrap().t,
            TokenType::Literal(Lit::LitFloat(0.2f64.to_bits()))
        );
        assert_eq!(
            lexer.next_token().unwrap().t,
            TokenType::Literal(Lit::LitFloat(235.0f64.to_bits()))
        );
        assert_eq!(
            lexer.next_token().unwrap().t,
            TokenType::Literal(Lit::LitFloat(214.0f64.to_bits()))
        );
        assert_eq!(
            lexer.next_token().unwrap().t,
            TokenType::Literal(Lit::LitFloat(0.4f64.to_bits()))
        );
        assert_eq!(
            lexer.next_token().unwrap().t,
            TokenType::Literal(Lit::LitStr("string".into()))
        );
        assert_eq!(
            lexer.next_token().unwrap().t,
            TokenType::Literal(Lit::LitBool(true))
        );
        assert_eq!(
            lexer.next_token().unwrap().t,
            TokenType::Literal(Lit::LitBool(false))
        );
    }
}
