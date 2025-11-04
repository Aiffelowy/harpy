use std::fmt::Display;

use crate::lexer::tokens::Ident;
use crate::t;
use crate::tt;

use super::parse_trait::Parse;
use super::parser::Parser;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomType(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BaseType {
    Primitive(PrimitiveType),
    Custom(CustomType),
}

impl Parse for BaseType {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        if let Some(t) = parser.try_parse::<PrimitiveType>() {
            return Ok(Self::Primitive(t));
        }

        let type_ident = parser.consume::<Ident>()?;
        return Ok(Self::Custom(CustomType(type_ident.value().clone())));
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeInner {
    Base(BaseType),
    Boxed(Box<Type>),
    Ref(Box<Type>),
    Void,
    Unknown,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
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

    pub fn calc_size(&self) -> usize {
        let mut size = 0;

        match &self.inner {
            TypeInner::Ref(_) => size += 8,
            TypeInner::Boxed(_) => size += 8,
            TypeInner::Void => (),
            TypeInner::Unknown => (),
            TypeInner::Base(b) => match b {
                BaseType::Custom(_) => (),
                BaseType::Primitive(p) => match p {
                    PrimitiveType::Int => size += 8,
                    PrimitiveType::Str => (),
                    PrimitiveType::Bool => size += 1,
                    PrimitiveType::Float => size += 8,
                },
            },
        }

        size
    }

    pub fn compatible(&self, other: &Type) -> bool {
        match (&self.inner, &other.inner) {
            (TypeInner::Base(lhs), TypeInner::Base(rhs)) => {
                lhs == rhs && (!other.mutable || self.mutable)
            }

            (TypeInner::Boxed(lhs), TypeInner::Boxed(rhs)) => {
                (!other.mutable || self.mutable) && lhs.compatible(rhs)
            }

            (TypeInner::Ref(lhs), TypeInner::Ref(rhs)) => {
                if self.mutable {
                    rhs.mutable && lhs.compatible(rhs)
                } else {
                    lhs.compatible(rhs)
                }
            }

            (TypeInner::Void, TypeInner::Void) => true,
            _ => false,
        }
    }

    pub fn compatible_less_strict(&self, other: &Type) -> bool {
        match (&self.inner, &other.inner) {
            // Base types: ignore mutability
            (TypeInner::Base(lhs), TypeInner::Base(rhs)) => lhs == rhs,

            // Boxed types: check inner type recursively, ignore outer mut
            (TypeInner::Boxed(lhs), TypeInner::Boxed(rhs)) => lhs.compatible_less_strict(rhs),

            // References: LHS &T can accept any RHS; &mut T requires RHS to be mutable
            (TypeInner::Ref(lhs), TypeInner::Ref(rhs)) => {
                if self.mutable {
                    rhs.mutable && lhs.compatible_less_strict(rhs)
                } else {
                    lhs.compatible_less_strict(rhs)
                }
            }

            // Void types must match
            (TypeInner::Void, TypeInner::Void) => true,

            _ => false,
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
            BaseType::Custom(i) => &i.0,
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
