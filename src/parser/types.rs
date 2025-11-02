use crate::lexer::tokens::Ident;
use crate::t;
use crate::tt;

use super::parse_trait::Parse;
use super::parser::Parser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveType {
    Int,
    Str,
    Float,
    Bool,
}

impl Parse for PrimitiveType {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        let s = match parser.peek()? {
            tt!(int) => {
                parser.consume::<t!(int)>()?;
                Self::Int
            }
            tt!(float) => {
                parser.consume::<t!(float)>()?;
                Self::Float
            }
            tt!(str) => {
                parser.consume::<t!(str)>()?;
                Self::Str
            }
            tt!(bool) => {
                parser.consume::<t!(bool)>()?;
                Self::Bool
            }
            _ => {
                return parser.unexpected("primitive type");
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
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        if let Some(t) = parser.try_parse::<PrimitiveType>() {
            return Ok(Self::Primitive(t));
        }

        return Ok(Self::Custom(parser.consume::<Ident>()?));
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeInner {
    Base(BaseType),
    Boxed(Box<Type>),
    Ref(Box<Type>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub mutable: bool,
    pub inner: TypeInner,
}

impl Parse for Type {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        let mut mutable = false;

        if let tt!(&) = parser.peek()? {
            parser.consume::<t!(&)>()?;
            return Ok(Self {
                inner: TypeInner::Ref(Box::new(parser.parse::<Type>()?)),
                mutable: false,
            });
        }

        if let tt!(mut) = parser.peek()? {
            mutable = true;
            parser.consume::<t!(mut)>()?;
        }

        let inner = if let tt!(boxed) = parser.peek()? {
            parser.consume::<t!(boxed)>()?;
            TypeInner::Boxed(Box::new(parser.parse::<Type>()?))
        } else {
            TypeInner::Base(parser.parse::<BaseType>()?)
        };

        Ok(Self { mutable, inner })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::Lexer,
        parser::{
            parser::Parser,
            types::{BaseType, PrimitiveType, TypeInner},
        },
    };

    use super::Type;

    #[test]
    fn test_type_parsing() {
        let mut parser = Parser::new(Lexer::new("mut boxed int").unwrap());
        parser.parse::<Type>().unwrap();
    }

    #[test]
    fn test_nested_pointer_parsing() {
        let mut parser = Parser::new(Lexer::new("&boxed boxed &mut boxed mut str").unwrap());
        let t = parser.parse::<Type>().unwrap();
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
