use super::{BaseType, PrimitiveType};
use crate::parser::{parser::Parser, Parse};
use crate::{t, tt};
use std::fmt::Display;

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

        let inner = match parser.peek()? {
            tt!(boxed) => {
                parser.consume::<t!(boxed)>()?;
                TypeInner::Boxed(Box::new(parser.parse::<Type>()?))
            }
            tt!(.) => {
                parser.consume::<t!(.)>()?;
                TypeInner::Unknown
            }
            _ => TypeInner::Base(parser.parse()?),
        };

        Ok(Self { mutable, inner })
    }
}

impl Type {
    pub fn deref(&self) -> &Type {
        match &self.inner {
            TypeInner::Ref(inner) => inner.deref(),
            _ => self,
        }
    }

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

    pub fn float() -> Self {
        Self {
            mutable: false,
            inner: TypeInner::Base(BaseType::Primitive(PrimitiveType::Float)),
        }
    }

    pub fn bool() -> Self {
        Self {
            mutable: false,
            inner: TypeInner::Base(BaseType::Primitive(PrimitiveType::Bool)),
        }
    }


    pub fn str() -> Self {
        Self {
            mutable: false,
            inner: TypeInner::Base(BaseType::Primitive(PrimitiveType::Str)),
        }
    }

    pub fn void() -> Self {
        Self {
            mutable: false,
            inner: TypeInner::Void,
        }
    }

    pub fn boxed(ty: Type) -> Self {
        Self {
            mutable: false,
            inner: TypeInner::Boxed(Box::new(ty)),
        }
    }

    pub fn refed(ty: Type) -> Self {
        Self {
            mutable: false,
            inner: TypeInner::Ref(Box::new(ty)),
        }
    }

    pub fn is_ref(&self) -> bool {
        if let TypeInner::Ref(_) = &self.inner {
            return true;
        }
        false
    }

    pub fn calc_size(&self) -> u8 {
        match &self.inner {
            TypeInner::Ref(_) => 16,
            TypeInner::Boxed(_) => 16,
            TypeInner::Void => 0,
            TypeInner::Unknown => 0,
            TypeInner::Base(b) => match b {
                BaseType::Custom(_) => 0,
                BaseType::Primitive(p) => match p {
                    PrimitiveType::Int => 8,
                    PrimitiveType::Str => 16,
                    PrimitiveType::Bool => 1,
                    PrimitiveType::Float => 8,
                },
            },
        }
    }

    pub fn verify_pointers(&self) -> bool {
        match &self.inner {
            TypeInner::Boxed(b) => {
                if let TypeInner::Ref(_) = &b.inner {
                    return false;
                }
                b.verify_pointers()
            }
            TypeInner::Ref(r) => r.verify_pointers(),
            _ => true,
        }
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
                if l.mutable {
                    rhs.mutable && l.assign_compatible(r)
                } else {
                    l.assign_compatible(r)
                }
            }
            (TypeInner::Void, TypeInner::Void) => true,
            _ => false,
        }
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
    use super::*;

    #[test]
    fn test_type_compatibility_edge_cases() {
        let mut_int = Type { 
            mutable: true, 
            inner: TypeInner::Base(BaseType::Primitive(PrimitiveType::Int)) 
        };
        let immut_int = Type::int();
        
        assert!(mut_int.compatible(&immut_int), "mutable should be compatible with immutable");
        assert!(!immut_int.compatible(&mut_int), "immutable should not be compatible with mutable");
    }

    #[test]
    fn test_strict_compatibility() {
        let mut_int = Type { 
            mutable: true, 
            inner: TypeInner::Base(BaseType::Primitive(PrimitiveType::Int)) 
        };
        let immut_int = Type::int();
        
        assert!(!mut_int.strict_compatible(&immut_int), "strict compatibility requires exact mutability match");
        assert!(!immut_int.strict_compatible(&mut_int), "strict compatibility is symmetric");
        assert!(mut_int.strict_compatible(&mut_int), "same types should be strictly compatible");
        assert!(immut_int.strict_compatible(&immut_int), "same types should be strictly compatible");
    }

    #[test]
    fn test_param_compatibility() {
        let mut_ref_int = Type { 
            mutable: true, 
            inner: TypeInner::Ref(Box::new(
                    Type {
                        mutable: true,
                        inner: TypeInner::Base(BaseType::Primitive(PrimitiveType::Int))
                    }
            ))
        };
        let immut_ref_int = Type::refed(Type::int());
        
        assert!(!mut_ref_int.param_compatible(&immut_ref_int), "mutable ref param cannot accept immutable ref arg");
        assert!(immut_ref_int.param_compatible(&mut_ref_int), "immutable ref param can accept mutable ref arg");
    }

    #[test]
    fn test_return_compatibility() {
        let int_type = Type::int();
        let bool_type = Type::bool();
        
        assert!(int_type.return_compatible(&int_type), "same types should be return compatible");
        assert!(!int_type.return_compatible(&bool_type), "different types should not be return compatible");
    }

    #[test]
    fn test_assign_compatibility() {
        let mut_int = Type { 
            mutable: true, 
            inner: TypeInner::Base(BaseType::Primitive(PrimitiveType::Int)) 
        };
        let immut_int = Type::int();
        
        assert!(mut_int.assign_compatible(&immut_int), "can assign immutable to mutable");
        assert!(immut_int.assign_compatible(&immut_int), "can assign same type");
    }

    #[test]
    fn test_reference_mutability() {
        let mut_ref_mut_int = Type { 
            mutable: false, 
            inner: TypeInner::Ref(Box::new(Type { 
                mutable: true, 
                inner: TypeInner::Base(BaseType::Primitive(PrimitiveType::Int)) 
            })) 
        };
        let immut_ref_immut_int = Type::refed(Type::int());
        
        assert!(!mut_ref_mut_int.assign_compatible(&immut_ref_immut_int), 
            "cannot assign &T to &mut T through reference");
    }

    #[test]
    fn test_boxed_type_compatibility() {
        let box_int = Type::boxed(Type::int());
        let box_bool = Type::boxed(Type::bool());
        let int_type = Type::int();
        
        assert!(box_int.compatible(&box_int), "boxed type compatible with itself");
        assert!(!box_int.compatible(&box_bool), "different boxed types not compatible");
        assert!(!box_int.compatible(&int_type), "boxed type not compatible with unboxed");
    }

    #[test]
    fn test_type_deref() {
        let ref_int = Type::refed(Type::int());
        let nested_ref = Type::refed(Type::refed(Type::int()));
        
        assert_eq!(ref_int.deref(), &Type::int());
        assert_eq!(nested_ref.deref(), &Type::int(), "deref should traverse all reference levels");
    }

    #[test]
    fn test_is_ref() {
        let ref_int = Type::refed(Type::int());
        let int_type = Type::int();
        let box_int = Type::boxed(Type::int());
        
        assert!(ref_int.is_ref(), "reference type should return true");
        assert!(!int_type.is_ref(), "non-reference type should return false");
        assert!(!box_int.is_ref(), "boxed type should return false");
    }

    #[test]
    fn test_verify_pointers() {
        let valid_box_int = Type::boxed(Type::int());
        let invalid_box_ref = Type::boxed(Type::refed(Type::int()));
        let valid_ref_box = Type::refed(Type::boxed(Type::int()));
        
        assert!(valid_box_int.verify_pointers(), "box<int> should be valid");
        assert!(!invalid_box_ref.verify_pointers(), "box<&T> should be invalid");
        assert!(valid_ref_box.verify_pointers(), "&box<T> should be valid");
    }
}
