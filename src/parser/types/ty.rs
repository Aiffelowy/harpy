use std::fmt::Display;

use super::{BaseType, PrimitiveType, RuntimeType};
use crate::aliases::Result;
use crate::err::HarpyError;
use crate::lexer::span::Span;
use crate::parser::{parser::Parser, Parse};
use crate::{t, tt};

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
                lhs == rhs && (self.mutable || !other.mutable)
            }

            (TypeInner::Boxed(lhs), TypeInner::Boxed(rhs)) => {
                (self.mutable || !other.mutable) && lhs.compatible(rhs)
            }

            (TypeInner::Ref(lhs), TypeInner::Ref(rhs)) => {
                (self.mutable || !other.mutable) && lhs.compatible(rhs)
            }

            (TypeInner::Void, TypeInner::Void) => true,
            _ => false,
        }
    }

    pub fn strict_compatible(&self, other: &Type) -> bool {
        match (&self.inner, &other.inner) {
            (TypeInner::Base(lhs), TypeInner::Base(rhs)) => {
                lhs == rhs && self.mutable == other.mutable
            }

            (TypeInner::Boxed(lhs), TypeInner::Boxed(rhs)) => {
                self.mutable == other.mutable && lhs.strict_compatible(rhs)
            }

            (TypeInner::Ref(lhs), TypeInner::Ref(rhs)) => {
                self.mutable == other.mutable && lhs.strict_compatible(rhs)
            }

            (TypeInner::Void, TypeInner::Void) => true,
            _ => false,
        }
    }

    pub fn param_compatible(&self, arg: &Type) -> bool {
        match (&self.inner, &arg.inner) {
            (TypeInner::Base(lhs), TypeInner::Base(rhs)) => lhs == rhs,

            (TypeInner::Boxed(lhs), TypeInner::Boxed(arg_inner))
            | (TypeInner::Ref(lhs), TypeInner::Ref(arg_inner)) => {
                if lhs.mutable && !arg_inner.mutable {
                    return false;
                }
                lhs.param_compatible(arg_inner)
            }

            (TypeInner::Void, TypeInner::Void) => true,
            _ => false,
        }
    }

    pub fn return_compatible(&self, other: &Type) -> bool {
        match (&self.inner, &other.inner) {
            (TypeInner::Base(lhs), TypeInner::Base(rhs)) => lhs == rhs,
            (TypeInner::Boxed(lhs), TypeInner::Boxed(rhs)) => lhs.strict_compatible(rhs),
            (TypeInner::Ref(lhs), TypeInner::Ref(rhs)) => lhs.strict_compatible(rhs),
            (TypeInner::Void, TypeInner::Void) => true,
            _ => false,
        }
    }

    pub fn assign_compatible(&self, rhs: &Type) -> bool {
        match (&self.inner, &rhs.inner) {
            (TypeInner::Base(l), TypeInner::Base(r)) => l == r,
            (TypeInner::Boxed(l), TypeInner::Boxed(r)) => l.assign_compatible(r),
            (TypeInner::Ref(l), TypeInner::Ref(r)) => {
                if self.mutable {
                    r.mutable && l.assign_compatible(r)
                } else {
                    l.assign_compatible(r)
                }
            }
            _ => false,
        }
    }

    pub fn to_runtime(&self) -> Result<RuntimeType> {
        let new = match &self.inner {
            TypeInner::Void => RuntimeType::Void,
            TypeInner::Unknown => {
                return HarpyError::semantic(
                    crate::semantic_analyzer::err::SemanticError::UnresolvedType,
                    Span::default(),
                )
            }
            TypeInner::Ref(t) => RuntimeType::Ref(Box::new(t.to_runtime()?)),
            TypeInner::Boxed(t) => RuntimeType::Boxed(Box::new(t.to_runtime()?)),
            TypeInner::Base(b) => RuntimeType::Base(b.clone()),
        };

        Ok(new)
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
