use crate::{
    aliases::Result,
    err::HarpyError,
    parser::{
        expr::infix::{InfixOp, InfixOpKind},
        types::{BaseType, PrimitiveType, Type, TypeInner},
    },
    semantic_analyzer::err::SemanticError,
};

pub struct InfixResolver;

impl InfixResolver {
    pub fn resolve(op: &InfixOp, lhs: &Type, rhs: &Type) -> Result<Type> {
        if !Self::validate(op, lhs, rhs) {
            return HarpyError::semantic(
                SemanticError::InfixTypeMismatch(op.clone(), lhs.clone(), rhs.clone()),
                op.span(),
            );
        }

        Ok(Self::result(op, lhs, rhs))
    }

    fn validate(op: &InfixOp, lhs: &Type, rhs: &Type) -> bool {
        let lhs = lhs.deref();
        let rhs = rhs.deref();

        if matches!(lhs.inner, TypeInner::Boxed(_)) || matches!(rhs.inner, TypeInner::Boxed(_)) {
            return false;
        }

        match (&lhs.inner, &rhs.inner) {
            (
                TypeInner::Base(BaseType::Primitive(lhs_p)),
                TypeInner::Base(BaseType::Primitive(rhs_p)),
            ) => match op.op {
                InfixOpKind::Plus | InfixOpKind::Minus | InfixOpKind::Mult | InfixOpKind::Div => {
                    matches!(
                        (lhs_p, rhs_p),
                        (PrimitiveType::Int, PrimitiveType::Int)
                            | (PrimitiveType::Float, PrimitiveType::Float)
                    )
                }

                InfixOpKind::Eq
                | InfixOpKind::Lt
                | InfixOpKind::Gt
                | InfixOpKind::GtEq
                | InfixOpKind::LtEq => {
                    matches!(
                        (lhs_p, rhs_p),
                        (PrimitiveType::Int, PrimitiveType::Int)
                            | (PrimitiveType::Float, PrimitiveType::Float)
                    )
                }

                InfixOpKind::And | InfixOpKind::Or => {
                    matches!((lhs_p, rhs_p), (PrimitiveType::Bool, PrimitiveType::Bool))
                }
            },

            _ => false,
        }
    }

    fn result(op: &InfixOp, lhs: &Type, _rhs: &Type) -> Type {
        let lhs = lhs.deref();

        match op.op {
            InfixOpKind::Plus | InfixOpKind::Minus | InfixOpKind::Mult | InfixOpKind::Div => {
                lhs.clone()
            }

            InfixOpKind::Eq
            | InfixOpKind::Lt
            | InfixOpKind::Gt
            | InfixOpKind::LtEq
            | InfixOpKind::GtEq
            | InfixOpKind::And
            | InfixOpKind::Or => Type::bool(),
        }
    }
}
