use std::fmt::Display;

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
    Void,
    Unknown,
}

#[derive(Debug, Clone)]
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

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        if !self.mutable {
            return self.inner == other.inner;
        }

        self.inner == other.inner
    }
}

impl Type {
    pub fn unknown() -> Self {
        Self {
            mutable: false,
            inner: TypeInner::Unknown,
        }
    }

    pub fn int() -> Self {
        Self {
            mutable: false,
            inner: TypeInner::Base(BaseType::Primitive(PrimitiveType::Int)),
        }
    }

    pub fn bool() -> Self {
        Self {
            mutable: false,
            inner: TypeInner::Base(BaseType::Primitive(PrimitiveType::Bool)),
        }
    }

    pub fn void() -> Self {
        Self {
            mutable: false,
            inner: TypeInner::Void,
        }
    }
}

impl Display for PrimitiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Int => "int",
            Self::Float => "float",
            Self::Str => "str",
            Self::Bool => "bool",
        };

        write!(f, "{s}")
    }
}

impl Display for BaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BaseType::Custom(i) => i.value(),
            BaseType::Primitive(p) => &p.to_string(),
        };

        write!(f, "{s}")
    }
}

impl Display for TypeInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TypeInner::Void => "void".to_owned(),
            TypeInner::Unknown => "unknown".to_owned(),
            TypeInner::Base(b) => b.to_string(),
            TypeInner::Boxed(b) => format!("boxed {b}"),
            TypeInner::Ref(r) => format!("&{r}"),
        };

        write!(f, "{s}")
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = format!(
            "{}{}",
            if self.mutable { "mut " } else { "" },
            self.inner.to_string()
        );

        write!(f, "{s}")
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
