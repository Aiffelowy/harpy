use crate::{
    aliases::Result,
    err::HarpyError,
    parser::{expr::prefix::PrefixOp, types::Type},
    semantic_analyzer::err::SemanticError,
};

mod refr {
    use crate::parser::{
        expr::prefix::{PrefixOp, PrefixOpKind},
        types::Type,
    };

    use super::ttype;

    pub(super) fn validate(op: &PrefixOp, ttype: &Type) -> bool {
        match op.op {
            PrefixOpKind::Star => true,
            _ => ttype::validate(op, ttype),
        }
    }
}

mod boxed {
    use crate::parser::expr::prefix::{PrefixOp, PrefixOpKind};

    pub(super) fn validate(op: &PrefixOp) -> bool {
        match op.op {
            PrefixOpKind::Star => true,
            _ => false,
        }
    }
}

#[allow(unused)]
mod custom {
    use crate::{
        lexer::tokens::Ident,
        parser::{
            expr::prefix::PrefixOp,
            types::{CustomType, PrimitiveType, Type},
        },
    };

    pub(super) fn validate(op: &PrefixOp, ttype: &CustomType) -> bool {
        false
    }
}

mod primitive {
    use crate::parser::{
        expr::prefix::{PrefixOp, PrefixOpKind},
        types::PrimitiveType,
    };

    type TType = PrimitiveType;

    pub(super) fn validate(op: &PrefixOp, ttype: &TType) -> bool {
        match op.op {
            PrefixOpKind::Star => false,
            PrefixOpKind::Neg => match ttype {
                PrimitiveType::Int => true,
                PrimitiveType::Float => true,
                _ => false,
            },
            PrefixOpKind::Plus => match ttype {
                PrimitiveType::Int => true,
                PrimitiveType::Float => true,
                _ => false,
            },
            PrefixOpKind::Minus => match ttype {
                PrimitiveType::Int => true,
                PrimitiveType::Float => true,
                _ => false,
            },
        }
    }
}

mod base {
    use crate::parser::{expr::prefix::PrefixOp, types::BaseType};

    type Type = BaseType;

    pub(super) fn validate(op: &PrefixOp, ttype: &Type) -> bool {
        match ttype {
            BaseType::Primitive(p) => super::primitive::validate(op, p),
            BaseType::Custom(c) => super::custom::validate(op, c),
        }
    }
}

mod type_inner {

    use crate::parser::{expr::prefix::PrefixOp, types::TypeInner};

    use super::{base, boxed, refr};

    type Type = TypeInner;

    pub(super) fn validate(op: &PrefixOp, ttype: &Type) -> bool {
        match ttype {
            TypeInner::Base(b) => base::validate(op, b),
            TypeInner::Boxed(_) => boxed::validate(op),
            TypeInner::Ref(t) => refr::validate(op, t),
            TypeInner::Unknown => false,
            TypeInner::Void => false,
        }
    }
}

mod ttype {
    use crate::parser::{
        expr::prefix::{PrefixOp, PrefixOpKind},
        types::TypeInner,
    };

    use super::type_inner;

    type Type = crate::parser::types::Type;

    pub(super) fn validate(op: &PrefixOp, ttype: &Type) -> bool {
        type_inner::validate(op, &ttype.inner)
    }

    pub(super) fn result(op: &PrefixOp, ttype: &Type) -> Type {
        match op.op {
            PrefixOpKind::Star => match &ttype.inner {
                TypeInner::Base(_) => unreachable!(),
                TypeInner::Boxed(t) => *t.clone(),
                TypeInner::Ref(t) => *t.clone(),
                TypeInner::Unknown => unreachable!(),
                TypeInner::Void => unreachable!(),
            },

            _ => ttype.clone(),
        }
    }
}

pub struct PrefixResolver;

impl PrefixResolver {
    pub fn resolve(op: &PrefixOp, ttype: &Type) -> Result<Type> {
        if !ttype::validate(op, ttype) {
            return HarpyError::semantic(
                SemanticError::PrefixTypeMismatch(op.clone(), ttype.clone()),
                op.span(),
            );
        }

        Ok(ttype::result(op, ttype))
    }
}
