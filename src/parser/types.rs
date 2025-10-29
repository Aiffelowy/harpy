use crate::lexer::err::LexerError;

use crate::lexer::tokens::Ident;
use crate::t;
use crate::tt;

use super::parse_trait::Parse;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveType {
    Int,
    Str,
    Float,
    Bool,
}

impl Parse for PrimitiveType {
    fn parse(token_stream: &mut crate::lexer::Lexer) -> crate::aliases::Result<Self> {
        let s = match token_stream.peek()? {
            tt!(int) => {
                token_stream.consume::<t!(int)>()?;
                Self::Int
            }
            tt!(float) => {
                token_stream.consume::<t!(float)>()?;
                Self::Float
            }
            tt!(str) => {
                token_stream.consume::<t!(str)>()?;
                Self::Str
            }
            tt!(bool) => {
                token_stream.consume::<t!(bool)>()?;
                Self::Bool
            }
            _ => {
                return Err(LexerError::UnexpectedToken(
                    "primitive type",
                    token_stream.next_token()?,
                )
                .into())
            }
        };

        Ok(s)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BaseType {
    Primitive(PrimitiveType),
    Custom(Ident),
}

impl Parse for BaseType {
    fn parse(token_stream: &mut crate::lexer::Lexer) -> crate::aliases::Result<Self> {
        if let Some(t) = token_stream.try_parse::<PrimitiveType>() {
            return Ok(Self::Primitive(t));
        }

        return Ok(Self::Custom(token_stream.consume::<Ident>()?));
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeInner {
    Base(BaseType),
    Boxed(Box<Type>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub mutable: bool,
    pub reference: bool,
    pub inner: TypeInner,
}

impl Parse for Type {
    fn parse(token_stream: &mut crate::lexer::Lexer) -> crate::aliases::Result<Self> {
        let mut mutable = false;
        let mut reference = false;

        if let tt!(&) = token_stream.peek()? {
            reference = true;
            token_stream.consume::<t!(&)>()?;
        }

        if let tt!(mut) = token_stream.peek()? {
            mutable = true;
            token_stream.consume::<t!(mut)>()?;
        }

        let inner = if let tt!(boxed) = token_stream.peek()? {
            token_stream.consume::<t!(boxed)>()?;
            TypeInner::Boxed(Box::new(token_stream.parse::<Type>()?))
        } else {
            TypeInner::Base(token_stream.parse::<BaseType>()?)
        };

        Ok(Self {
            mutable,
            reference,
            inner,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::Lexer,
        parser::types::{BaseType, PrimitiveType, TypeInner},
    };

    use super::Type;

    #[test]
    fn test_type_parsing() {
        let mut lexer = Lexer::new("mut boxed int").unwrap();
        lexer.parse::<Type>().unwrap();
    }

    #[test]
    fn test_nested_pointer_parsing() {
        let mut lexer = Lexer::new("&boxed boxed &mut boxed mut str").unwrap();
        let t = lexer.parse::<Type>().unwrap();
        assert_eq!(
            t,
            Type {
                mutable: false,
                reference: true,
                inner: TypeInner::Boxed(Box::new(Type {
                    mutable: false,
                    reference: false,
                    inner: TypeInner::Boxed(Box::new(Type {
                        mutable: true,
                        reference: true,
                        inner: TypeInner::Boxed(Box::new(Type {
                            mutable: true,
                            reference: false,
                            inner: TypeInner::Base(BaseType::Primitive(PrimitiveType::Str))
                        }))
                    }))
                }))
            }
        )
    }
}
