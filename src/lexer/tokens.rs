use crate::aliases::Result;
use crate::lexer::err::LexerError;
use crate::lexer::span::Span;
use crate::lexer::Lexer;

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
    };
}

macro_rules! define_symbols_enum {
    ($( $symbol_char:literal => $symbol_ident:ident { $( $follow_up_char:literal => $follow_up_ident:ident, )* } )+) => {
        #[allow(unused)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Sym {
            $($symbol_ident, $($follow_up_ident,)*)+
        }
    };
}

macro_rules! define_literals_enum {
    ($( $literal_ident:ident $(($literal_type:ident))? )+) => {
        #[allow(unused)]
        #[derive(Debug, Clone, PartialEq)]
        pub enum Lit {
            $($literal_ident$(($literal_type))?,)+
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

                return Err(LexerError::UnexpectedToken(stringify!($name), token).into());
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
        [literals] => { $( $literal_ident:ident ($literal_type:ident) )+ }
    ) => {

        define_keywords_enum!($($keyword_lit => $keyword_ident,)+);
        define_symbols_enum!($( $symbol_char => $symbol_ident { $($follow_up_char => $follow_up_ident,)* } )+);
        define_literals_enum!($( $literal_ident ($literal_type) )+);

        #[allow(unused)]
        #[derive(Debug, Clone, PartialEq)]
        pub enum TokenType {
            Keyword(Key),
            Symbol(Sym),
            Literal(Lit),
            Ident(String),
            Eof,
        }

        #[allow(unused)]
        #[derive(Debug, Clone, PartialEq)]
        pub struct Token {
            pub(in crate::lexer) t: TokenType,
            pub(in crate::lexer) span: Span
        }

        impl Token {
            pub fn kind(&self) -> &TokenType { &self.t }
            fn parse_number(l: &mut Lexer) -> Result<TokenType> {
                let mut result = String::with_capacity(4);
                let mut is_float = false;

                while let Some(c) = l.peek_char()? {
                    match c {
                        '0'..='9' => {
                            result.push(c);
                            l.next_char()?;
                        }
                        '.' => {
                            if is_float {
                                break;
                            }
                            is_float = true;
                            result.push(c);
                            l.next_char()?;
                        }
                        'f' | 'F' => {
                            is_float = true;
                            l.next_char()?;
                            break;
                        }
                        _ => break,
                    }
                }

                if is_float {
                    let value: f64 = result.parse().map_err(|e| LexerError::InvalidFloat(e))?;
                    return Ok(TokenType::Literal(Lit::LitFloat(value)))
                }

                let value: u64 = result.parse().map_err(|e| LexerError::InvalidInt(e))?;
                Ok(TokenType::Literal(Lit::LitInt(value)))
            }

            fn parse_ident(l: &mut Lexer) -> Result<TokenType> {
                let mut result = String::with_capacity(6);

                while let Some(c) = l.peek_char()? {
                    if !(c.is_alphanumeric() || c == '_') { break; }

                    l.next_char()?;
                    result.push(c);

                }

                match result.as_str() {
                    $( $keyword_lit => return Ok(TokenType::Keyword(Key::$keyword_ident)), )+
                    "false" => return Ok(TokenType::Literal(Lit::LitBool(false))),
                    "true" => return Ok(TokenType::Literal(Lit::LitBool(true))),
                    _ => return Ok(TokenType::Ident(result))
                }
            }

            fn parse_str(l: &mut Lexer) -> Result<TokenType> {
                l.next_char()?; //discard first "
                let mut result = String::with_capacity(10);
                loop {
                    let Some(c) = l.next_char()? else { return Err(LexerError::UnclosedStr(Span::new(l.position(), l.position())).into()) };
                    if c == '"' { break; }
                    result.push(c);
                }

                return Ok(TokenType::Literal(Lit::LitStr(result)))
            }


            pub(super) fn parse(l: &mut Lexer) -> Result<Self> {
                l.skip_whitespace()?;
                let position_start = l.position();
                let Some(c) = l.peek_char()? else { return Ok(Self { t: TokenType::Eof, span: Span::new(position_start, position_start) }) };

                let token_type =

                if c.is_numeric() {
                    Self::parse_number(l)?
                } else

                if c == '_' || c.is_alphabetic() {
                    Self::parse_ident(l)?
                } else {

                match c {
                    $(
                        $symbol_char => {
                            l.next_char()?;
                            match l.peek_char()? {
                                $(Some($follow_up_char) => { l.next_char()?; TokenType::Symbol(Sym::$follow_up_ident)})*
                                _ => TokenType::Symbol(Sym::$symbol_ident)
                            }
                        },
                    )+
                    '"' => Self::parse_str(l)?,
                    _ => return Err(LexerError::UnknownToken(Span::new(position_start, l.position())).into()),
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
            define_token_struct!($literal_ident, { Literal(Lit::$literal_ident(value)) }, value: $literal_type);
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

        "boxed" => Boxed,
        "box" => Box,
        "amogus" => Amogus,
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
        }

        '/' => Slash {
            '=' => DivAssign,
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

        '!' => Neg {}

        ',' => Comma {}
    }

    [literals] => {
        LitInt(u64)
        LitFloat(f64)
        LitStr(String)
        LitBool(bool)
    }
);

#[macro_export]
macro_rules! t {
    (let) => {
        $crate::lexer::tokens::Let
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
    (boxed) => {
        $crate::lexer::tokens::Boxed
    };
    (box) => {
        $crate::lexer::tokens::Box
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
    (boxed) => {
        $crate::lexer::tokens::TokenType::Keyword($crate::lexer::tokens::Key::Boxed)
    };
    (box) => {
        $crate::lexer::tokens::TokenType::Keyword($crate::lexer::tokens::Key::Box)
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
