use crate::aliases::Result;
use crate::err::HarpyError;
use crate::lexer::err::LexerError;
use crate::lexer::span::Span;
use crate::lexer::Lexer;
use std::fmt::Display;

pub trait Tokenize
where
    Self: Sized,
{
    fn tokenize(token_stream: &mut Lexer) -> Result<Self>;
}

macro_rules! define_keywords_enum {
    ($( $keyword_lit:literal => $keyword_ident:ident, )+) => {
        #[allow(unused)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Key {
            $($keyword_ident,)+
        }

        impl Display for Key {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", match self {
                    $(Key::$keyword_ident => $keyword_lit,)+
                })
            }
        }
    };
}

macro_rules! define_symbols_enum {
    ($( $symbol_char:literal => $symbol_ident:ident { $( $follow_up_char:literal => $follow_up_ident:ident, )* } )+) => {
        #[allow(unused)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Sym {
            $($symbol_ident, $($follow_up_ident,)*)+
        }

        impl Display for Sym {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", match self {
                    $(
                        Sym::$symbol_ident => $symbol_char,
                        $(Sym::$follow_up_ident => $follow_up_char,)*
                    )+
                })
            }
        }
    };
}

macro_rules! define_literals_enum {
    ($( $literal_ident:ident ( $($literal_type:tt)* ) )+) => {
        #[allow(unused)]
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum Lit {
            $($literal_ident($($literal_type)*),)+
            LitVoid
        }

        impl Display for Lit {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", match self {
                    $(Lit::$literal_ident(l) => l.to_string(),)+
                    Lit::LitVoid => "()".to_owned(),
                })
            }
        }
    };
}

macro_rules! define_token_struct {
    ($name:ident, { $($token_type:tt)+ } $(,$value:ident: $($type:tt)+)?) => {
        #[allow(unused)]
        #[derive(Debug, Clone, PartialEq)]
        pub struct $name {
            span: Span,
            $($value: $($type)+)?
        }

        impl Tokenize for $name {
            fn tokenize(token_stream: &mut Lexer) -> Result<Self> {
                let token = token_stream.next_token()?;
                if let TokenType::$($token_type)+ = token.t {
                    return Ok(Self { span: token.span, $($value)? });
                }
                let span = token.span.clone();
                return HarpyError::lexer(LexerError::UnexpectedToken(stringify!($name), token), span);
            }
        }

        impl $name {
            $(pub fn value(&self) -> &$($type)+ { &self.$value })?
            pub fn span(&self) -> Span { self.span }
        }
    };
}

macro_rules! define_tokens {
    (
        [keywords] => { $( $keyword_lit:literal => $keyword_ident:ident, )+ }
        [symbols] => { $( $symbol_char:literal => $symbol_ident:ident { $( $follow_up_char:literal => $follow_up_ident:ident, )* } )+ }
        [literals] => { $( $literal_ident:ident ( $($literal_type:tt)* ) )+ }
    ) => {

        define_keywords_enum!($($keyword_lit => $keyword_ident,)+);
        define_symbols_enum!($( $symbol_char => $symbol_ident { $($follow_up_char => $follow_up_ident,)* } )+);
        define_literals_enum!($( $literal_ident ($($literal_type)*) )+);

        #[allow(unused)]
        #[derive(Debug, Clone, PartialEq)]
        pub enum TokenType {
            Keyword(Key),
            Symbol(Sym),
            Literal(Lit),
            Ident(String),
            Eof,
        }

        impl Display for TokenType {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", match self {
                    Self::Keyword(k) => k.to_string(),
                    Self::Symbol(s) => s.to_string(),
                    Self::Literal(l) => l.to_string(),
                    Self::Ident(_) => "identifier".to_string(),
                    Self::Eof => "eof".to_string()
                })
            }
        }

        #[allow(unused)]
        #[derive(Debug, Clone, PartialEq)]
        pub struct Token {
            pub(in crate::lexer) t: TokenType,
            pub(in crate::lexer) span: Span
        }

        impl Token {
            pub fn kind(&self) -> &TokenType { &self.t }
            pub fn span(&self) -> Span { self.span }
            fn parse_number(l: &mut Lexer) -> Result<TokenType> {
                let start = l.position();
                let mut result = String::with_capacity(4);
                let mut is_float = false;

                while let Some(c) = l.peek_char() {
                    match c {
                        '0'..='9' => {
                            result.push(c);
                            l.next_char();
                        }
                        '.' => {
                            if is_float {
                                break;
                            }
                            is_float = true;
                            result.push(c);
                            l.next_char();
                        }
                        'f' | 'F' => {
                            is_float = true;
                            l.next_char();
                            break;
                        }
                        _ => break,
                    }
                }

                let span = Span::new(start, l.position());

                if is_float {
                    let value: f64 = match result.parse() {
                        Ok(f) => f,
                        Err(e) => return HarpyError::lexer(LexerError::InvalidFloat(e), span),
                    };
                    return Ok(TokenType::Literal(Lit::LitFloat(value.to_bits())))
                }

                let value: u64 = match result.parse() {
                    Ok(i) => i,
                    Err(e) => return HarpyError::lexer(LexerError::InvalidInt(e), span)
                };
                Ok(TokenType::Literal(Lit::LitInt(value)))
            }

            fn parse_ident(l: &mut Lexer) -> TokenType {
                let mut result = String::with_capacity(6);

                while let Some(c) = l.peek_char() {
                    if !(c.is_alphanumeric() || c == '_') { break; }

                    l.next_char();
                    result.push(c);

                }

                match result.as_str() {
                    $( $keyword_lit => return TokenType::Keyword(Key::$keyword_ident), )+
                    "false" => return TokenType::Literal(Lit::LitBool(false)),
                    "true" => return TokenType::Literal(Lit::LitBool(true)),
                    _ => return TokenType::Ident(result)
                }
            }

            fn parse_str(l: &mut Lexer) -> Result<TokenType> {
                l.next_char(); //discard first "
                let mut result = String::with_capacity(10);
                loop {
                    let Some(c) = l.next_char() else {
                        return HarpyError::lexer(LexerError::UnclosedStr, Span::new(l.position(), l.position()));
                    };
                    if c == '"' { break; }
                    result.push(c);
                }

                return Ok(TokenType::Literal(Lit::LitStr(result)))
            }


            pub(super) fn parse(l: &mut Lexer) -> Result<Self> {
                l.skip_whitespace();
                let position_start = l.position();
                let Some(c) = l.peek_char() else { return Ok(Self { t: TokenType::Eof, span: Span::new(position_start, position_start) }) };

                let token_type =

                if c.is_numeric() {
                    Self::parse_number(l)?
                } else

                if c == '_' || c.is_alphabetic() {
                    Self::parse_ident(l)
                } else {

                match c {
                    $(
                        $symbol_char => {
                            l.next_char();
                            match l.peek_char() {
                                $(Some($follow_up_char) => { l.next_char(); TokenType::Symbol(Sym::$follow_up_ident)})*
                                _ => TokenType::Symbol(Sym::$symbol_ident)
                            }
                        },
                    )+
                    '"' => Self::parse_str(l)?,
                    _ => return HarpyError::lexer(LexerError::UnknownToken, Span::new(position_start, l.position())),
                }};
                Ok(Self { t: token_type, span: Span::new(position_start, l.position()) })

            }
        }


        $(
            define_token_struct!($keyword_ident, { Keyword(Key::$keyword_ident) });
        )+

        $(
            define_token_struct!($symbol_ident, { Symbol(Sym::$symbol_ident) });

        $(
            define_token_struct!($follow_up_ident, { Symbol(Sym::$follow_up_ident) });
        )*
        )+

        $(
            define_token_struct!($literal_ident, { Literal(Lit::$literal_ident(value)) }, value: $($literal_type)*);
        )+

        define_token_struct!(Ident, { Ident(value) }, value: String);
        define_token_struct!(Literal, { Literal(value) }, value: Lit);
        define_token_struct!(Keyword, { Keyword(value) }, value: Key);
        define_token_struct!(Symbol, { Symbol(value) }, value: Sym);

    };
}

define_tokens!(
    [keywords] => {
        "let" => Let,
        "global" => Global,
        "mut" => Mut,

        "fn" => Fn,
        "return" => Return,

        "int" => Int,
        "float" => Float,
        "str" => Str,
        "bool" => Bool,

        "for" => For,
        "in" => In,
        "while" => While,
        "loop" => Loop,

        "if" => If,
        "else" => Else,
        "switch" => Switch,

        "boxed" => Boxed,
        "box" => Box,

        "borrowed" => Borrowed,
        "borrow" => Borrow,
    }

    [symbols] => {
        '=' => Assign {
            '>' => FatArrow,
            '=' => Compare,
        }

        '+' => Plus {
            '=' => PlusAssign,
        }

        '-' => Minus {
            '=' => MinusAssign,
            '>' => Arrow,
        }

        '*' => Star {
            '=' => MultAssign,
            '/' => BlockCommentStop,
        }

        '/' => Slash {
            '=' => DivAssign,
            '/' => LineComment,
            '*' => BlockComment,
        }

        '>' => Gt {
            '=' => GtEq,
        }

        '<' => Lt {
            '=' => LtEq,
        }

        '(' => LParen {}
        ')' => RParen {}

        '{' => LCurly {}
        '}' => RCurly {}

        '[' => LSquare {}
        ']' => RSquare {}

        '.' => Dot {
            '.' => Range,
        }

        ':' => Colon {}

        ';' => Semi {}

        '&' => And {
            '&' => LogAnd,
        }

        '|' => Pipe {
            '|' => LogOr,
        }

        '!' => Neg {
            '=' => Neq,
        }

        ',' => Comma {}

        '%' => Modulo {
            '=' => ModuloAssign,
        }
    }

    [literals] => {
        LitInt(u64)
        LitFloat(u64)
        LitStr(String)
        LitBool(bool)
    }
);

#[macro_export]
macro_rules! t {
    (let) => {
        $crate::lexer::tokens::Let
    };
    (global) => {
        $crate::lexer::tokens::Global
    };
    (mut) => {
        $crate::lexer::tokens::Mut
    };
    (fn) => {
        $crate::lexer::tokens::Fn
    };
    (return) => {
        $crate::lexer::tokens::Return
    };
    (int) => {
        $crate::lexer::tokens::Int
    };
    (float) => {
        $crate::lexer::tokens::Float
    };
    (str) => {
        $crate::lexer::tokens::Str
    };
    (bool) => {
        $crate::lexer::tokens::Bool
    };
    (for) => {
        $crate::lexer::tokens::For
    };
    (in) => {
        $crate::lexer::tokens::In
    };
    (while) => {
        $crate::lexer::tokens::While
    };
    (loop) => {
        $crate::lexer::tokens::Loop
    };
    (if) => {
        $crate::lexer::tokens::If
    };
    (else) => {
        $crate::lexer::tokens::Else
    };
    (switch) => {
        $crate::lexer::tokens::Switch
    };
    (boxed) => {
        $crate::lexer::tokens::Boxed
    };
    (box) => {
        $crate::lexer::tokens::Box
    };
    (borrow) => {
        $crate::lexer::tokens::Borrow
    };
    (borrowed) => {
        $crate::lexer::tokens::Borrowed
    };
    (ident) => {
        $crate::lexer::tokens::Ident
    };
    (lit) => {
        $crate::lexer::tokens::Literal
    };

    (=) => {
        $crate::lexer::tokens::Assign
    };
    (=>) => {
        $crate::lexer::tokens::FatArrow
    };
    (==) => {
        $crate::lexer::tokens::Compare
    };
    (>) => {
        $crate::lexer::tokens::Gt
    };
    (<) => {
        $crate::lexer::tokens::Lt
    };
    (>=) => {
        $crate::lexer::tokens::GtEq
    };
    (<=) => {
        $crate::lexer::tokens::LtEq
    };
    (!=) => {
        $crate::lexer::tokens::Neq
    };
    (+) => {
        $crate::lexer::tokens::Plus
    };
    (+=) => {
        $crate::lexer::tokens::PlusAssign
    };
    (-) => {
        $crate::lexer::tokens::Minus
    };
    (-=) => {
        $crate::lexer::tokens::MinusAssign
    };
    (%) => {
        $crate::lexer::tokens::Modulo
    };
    (%=) => {
        $crate::lexer::tokens::ModuloAssign
    };
    (->) => {
        $crate::lexer::tokens::Arrow
    };
    (*) => {
        $crate::lexer::tokens::Star
    };
    (*=) => {
        $crate::lexer::tokens::MultAssign
    };
    (/) => {
        $crate::lexer::tokens::Slash
    };
    ("//") => {
        $crate::lexer::tokens::LineComment
    };
    ("/*") => {
        $crate::lexer::tokens::BlockComment
    };
    ("*/") => {
        $crate::lexer::tokens::BlockCommentStop
    };
    (/=) => {
        $crate::lexer::tokens::DivAssign
    };

    ("{") => {
        $crate::lexer::tokens::LCurly
    };
    ("}") => {
        $crate::lexer::tokens::RCurly
    };
    ("[") => {
        $crate::lexer::tokens::LSquare
    };
    ("]") => {
        $crate::lexer::tokens::RSquare
    };
    ("(") => {
        $crate::lexer::tokens::LParen
    };
    (")") => {
        $crate::lexer::tokens::RParen
    };
    (.) => {
        $crate::lexer::tokens::Dot
    };
    (:) => {
        $crate::lexer::tokens::Colon
    };
    (;) => {
        $crate::lexer::tokens::Semi
    };
    (ident) => {
        $crate::lexer::tokens::Ident
    };
    (&) => {
        $crate::lexer::tokens::And
    };
    (!) => {
        $crate::lexer::tokens::Neg
    };
    (&) => {
        $crate::lexer::tokens::And
    };
    (&&) => {
        $crate::lexer::tokens::LogAnd
    };
    (|) => {
        $crate::lexer::tokens::Pipe
    };
    (||) => {
        $crate::lexer::tokens::LogOr
    };
    (,) => {
        $crate::lexer::tokens::Comma
    };
    (..) => {
        $crate::lexer::tokens::Range
    };
}

#[macro_export]
macro_rules! tt {
    (let) => {
        $crate::lexer::tokens::TokenType::Keyword($crate::lexer::tokens::Key::Let)
    };
    (global) => {
        $crate::lexer::tokens::TokenType::Keyword($crate::lexer::tokens::Key::Global)
    };
    (mut) => {
        $crate::lexer::tokens::TokenType::Keyword($crate::lexer::tokens::Key::Mut)
    };
    (fn) => {
        $crate::lexer::tokens::TokenType::Keyword($crate::lexer::tokens::Key::Fn)
    };
    (return) => {
        $crate::lexer::tokens::TokenType::Keyword($crate::lexer::tokens::Key::Return)
    };
    (int) => {
        $crate::lexer::tokens::TokenType::Keyword($crate::lexer::tokens::Key::Int)
    };
    (float) => {
        $crate::lexer::tokens::TokenType::Keyword($crate::lexer::tokens::Key::Float)
    };
    (str) => {
        $crate::lexer::tokens::TokenType::Keyword($crate::lexer::tokens::Key::Str)
    };
    (bool) => {
        $crate::lexer::tokens::TokenType::Keyword($crate::lexer::tokens::Key::Bool)
    };
    (for) => {
        $crate::lexer::tokens::TokenType::Keyword($crate::lexer::tokens::Key::For)
    };
    (in) => {
        $crate::lexer::tokens::TokenType::Keyword($crate::lexer::tokens::Key::In)
    };
    (while) => {
        $crate::lexer::tokens::TokenType::Keyword($crate::lexer::tokens::Key::While)
    };
    (loop) => {
        $crate::lexer::tokens::TokenType::Keyword($crate::lexer::tokens::Key::Loop)
    };
    (if) => {
        $crate::lexer::tokens::TokenType::Keyword($crate::lexer::tokens::Key::If)
    };
    (else) => {
        $crate::lexer::tokens::TokenType::Keyword($crate::lexer::tokens::Key::Else)
    };
    (switch) => {
        $crate::lexer::tokens::TokenType::Keyword($crate::lexer::tokens::Key::Switch)
    };
    (boxed) => {
        $crate::lexer::tokens::TokenType::Keyword($crate::lexer::tokens::Key::Boxed)
    };
    (box) => {
        $crate::lexer::tokens::TokenType::Keyword($crate::lexer::tokens::Key::Box)
    };
    (borrow) => {
        $crate::lexer::tokens::TokenType::Keyword($crate::lexer::tokens::Key::Borrow)
    };
    (borrowed) => {
        $crate::lexer::tokens::TokenType::Keyword($crate::lexer::tokens::Key::Borrowed)
    };

    (=) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::Assign)
    };
    (=>) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::FatArrow)
    };
    (==) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::Compare)
    };

    (+) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::Plus)
    };
    (+=) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::PlusAssign)
    };

    (-) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::Minus)
    };
    (-=) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::MinusAssign)
    };
    (%) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::Modulo)
    };
    (%=) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::ModuloAssign)
    };
    (->) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::Arrow)
    };

    (*) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::Star)
    };
    (*=) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::MultAssign)
    };

    (/) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::Slash)
    };
    ("//") => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::LineComment)
    };
    ("/*") => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::BlockComment)
    };
    ("*/") => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::BlockCommentStop)
    };
    (/=) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::DivAssign)
    };

    ("(") => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::LParen)
    };
    (")") => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::RParen)
    };
    ("{") => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::LCurly)
    };
    ("}") => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::RCurly)
    };
    ("[") => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::LSquare)
    };
    ("]") => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::RSquare)
    };
    (.) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::Dot)
    };
    (:) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::Colon)
    };
    (;) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::Semi)
    };
    (ident) => {
        $crate::lexer::tokens::TokenType::Ident(_)
    };
    (lit) => {
        $crate::lexer::tokens::TokenType::Literal(_)
    };
    (>) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::Gt)
    };
    (<) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::Lt)
    };
    (>=) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::GtEq)
    };
    (<=) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::LtEq)
    };
    (!=) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::Neq)
    };
    (&) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::And)
    };
    (!) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::Neg)
    };
    (&) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::And)
    };
    (&&) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::LogAnd)
    };
    (|) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::Pipe)
    };
    (||) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::LogOr)
    };
    (,) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::Comma)
    };
    (..) => {
        $crate::lexer::tokens::TokenType::Symbol($crate::lexer::tokens::Sym::Range)
    };
    (eof) => {
        $crate::lexer::tokens::TokenType::Eof
    };
}
