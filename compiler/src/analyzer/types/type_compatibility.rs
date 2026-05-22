use crate::{
    aliases::Result,
    analyzer::{
        analyzer::Analyzer,
        types::types::{ResolvedType, TypeId},
    },
    err::{HarpyError, Kind},
    lexer::span::Span,
    parser::node::NodeId,
};

impl Analyzer {
    pub(in crate::analyzer) fn unify(
        &mut self,
        expected: TypeId,
        actual: TypeId,
    ) -> Option<TypeId> {
        if expected == actual || actual == TypeId::never() || actual == TypeId::unknown() {
            return Some(expected);
        }
        if expected == TypeId::unknown() || expected == TypeId::never() {
            return Some(actual);
        }

        let exp_ty = expected.get(self).clone();
        let act_ty = actual.get(self).clone();

        match (exp_ty, act_ty) {
            (ResolvedType::Boxed(e_in, e_mut), ResolvedType::Boxed(a_in, a_mut)) => {
                let unified = self.unify(e_in, a_in)?;
                if e_mut && !a_mut {
                    return None;
                }
                let ty = ResolvedType::Boxed(unified, a_mut);
                Some(self.db.type_table.register(ty))
            }

            (ResolvedType::Ref(e_in, e_mut), ResolvedType::Ref(a_in, a_mut)) => {
                let unified = self.unify(e_in, a_in)?;
                if e_mut && !a_mut {
                    return None;
                }
                let ty = ResolvedType::Ref(unified, a_mut);
                Some(self.db.type_table.register(ty))
            }
            (ResolvedType::Array(e_in, e_size), ResolvedType::Array(a_in, a_size)) => {
                let unified = self.unify(e_in, a_in)?;

                let unified_size = match (e_size, a_size) {
                    (Some(s1), Some(s2)) => {
                        if s1 == s2 {
                            Some(s1)
                        } else {
                            return None;
                        }
                    }
                    (None, None) => None,
                    (None, Some(_)) | (Some(_), None) => None,
                };

                let ty = ResolvedType::Array(unified, unified_size);
                Some(self.db.type_table.register(ty))
            }
            _ => None,
        }
    }

    pub(in crate::analyzer::types) fn is_compatible(
        &self,
        expected: TypeId,
        actual: TypeId,
    ) -> bool {
        if expected == actual {
            return true;
        }
        if actual == TypeId::never() {
            return true;
        }

        if expected == TypeId::unknown() || actual == TypeId::unknown() {
            return true;
        }

        let expected = expected.get(self);
        let actual = actual.get(self);

        match (expected, actual) {
            (ResolvedType::Boxed(e_in, e_mut), ResolvedType::Boxed(a_in, a_mut)) => {
                if *e_mut && !a_mut {
                    return false;
                }
                self.is_compatible(*e_in, *a_in)
            }

            (ResolvedType::Ref(exp_inner, e_mut), ResolvedType::Boxed(act_inner, a_mut)) => {
                if *e_mut && !a_mut {
                    return false;
                }
                self.is_compatible(*exp_inner, *act_inner)
            }

            (ResolvedType::Ref(e_in, e_mut), ResolvedType::Ref(a_in, a_mut)) => {
                if *e_mut && !a_mut {
                    return false;
                }
                self.is_compatible(*e_in, *a_in)
            }

            (ResolvedType::Array(e_in, e_size), ResolvedType::Array(a_in, a_size)) => {
                if !self.is_compatible(*e_in, *a_in) {
                    return false;
                }

                match (e_size, a_size) {
                    (Some(s1), Some(s2)) => s1 == s2,
                    (None, _) => true,
                    (Some(_), None) => false,
                }
            }

            _ => false,
        }
    }

    pub(in crate::analyzer) fn ensure_compatible(
        &mut self,
        expected: TypeId,
        actual: TypeId,
        actual_expr_id: Option<NodeId>,
        span: Span,
    ) -> bool {
        if self.is_compatible(expected, actual) {
            return true;
        }
        if let Some(expr_id) = actual_expr_id {
            let mut current_ty = actual;
            let mut deref_count = 0;

            while let ResolvedType::Boxed(inner, _) | ResolvedType::Ref(inner, _) =
                current_ty.get(self)
            {
                current_ty = *inner;
                deref_count += 1;

                if self.is_compatible(expected, current_ty) {
                    self.db.auto_derefs.insert(expr_id, deref_count);
                    return true;
                }
            }
        }

        self.report_error(
            span,
            Kind::NotCompatible {
                expected,
                got: actual,
            },
        );
        false
    }

    pub(in crate::analyzer::types) fn get_iterable_yield_type(
        &self,
        iter_type: TypeId,
        span: Span,
    ) -> Result<TypeId> {
        let ty = iter_type.get(self);
        println!("{:?}", ty);
        match ty {
            ResolvedType::Array(inner, _) => Ok(*inner),
            ResolvedType::Range(inner) => Ok(*inner),
            _ => HarpyError::err(span, Kind::ExprNotIter),
        }
    }
}
