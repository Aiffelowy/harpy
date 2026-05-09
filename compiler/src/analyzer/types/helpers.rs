use crate::{analyzer::{analyzer::Analyzer, err::TypeCheckError}, lexer::span::Span, parser::{expr::expr_defs::Expr, node::NodeId, Node}};

use super::types::{ResolvedType, TypeId};



impl Analyzer {
    pub fn is_fully_resolved(&self, ty: TypeId) -> bool {
        if ty == TypeId::unknown() {
            return false;
        }

        match ty.get(self) {
            ResolvedType::Boxed(inner) => self.is_fully_resolved(*inner),
            ResolvedType::Ref(inner, _) => self.is_fully_resolved(*inner),
            ResolvedType::Array(inner, _) => self.is_fully_resolved(*inner),
            _ => true
        }
    }

    pub fn infer_type(&mut self, node: NodeId, ty_id: TypeId) {
        let Some(symbol) = self.db.symbol_table.get_symbol_mut(node) else {
            return;
        };

        symbol.ty = ty_id;
    }

    pub fn get_symbol_type(&mut self, node: NodeId) -> TypeId {
        self.db.symbol_table.get_symbol(node).ty
    }


    pub fn get_cached_type(&self, node: NodeId) -> TypeId {
        self.db
            .type_table
            .is_cached(node)
            .unwrap_or(TypeId::unknown())
    }

    pub fn auto_deref(&self, mut ty: TypeId) -> TypeId {
        loop {
            match ty.get(self) {
                ResolvedType::Boxed(inner) => ty = *inner,
                ResolvedType::Ref(inner, _) => ty = *inner,
                _ => return ty,
            }
        }
    }

    pub fn is_pointer(&self, ty: TypeId) -> bool {
        matches!(
            ty.get(self),
            ResolvedType::Boxed(_) | ResolvedType::Ref(_, _)
        )
    }

    pub fn is_lvalue(&self, expr: &Node<Expr>) -> bool {
        match &expr.inner {
            Expr::Ident(_) => true,
            Expr::MemberAccess(obj, _) => {
                let obj_ty = self.get_cached_type(obj.id);
                self.is_pointer(obj_ty) || self.is_lvalue(obj)
            }
            Expr::Index(array, _) => {
                let arr_ty = self.get_cached_type(array.id);
                self.is_pointer(arr_ty) || self.is_lvalue(array)
            }
            _ => false,
        }
    }

    pub fn check_binding(&mut self, expected: TypeId, actual: TypeId, value_node_id: Option<NodeId>, span: Span) -> TypeId {
        let unified_ty = self.unify(expected, actual).unwrap_or(expected);
        if let Some(id) = value_node_id {
            self.db.type_table.cache(id, unified_ty);
        }

        self.ensure_compatible(unified_ty, actual, span);

        if !self.is_fully_resolved(unified_ty) {
            self.report_error(TypeCheckError::TypeAnnotationsNeeded.into(), span);
        }

        unified_ty
    }

}
