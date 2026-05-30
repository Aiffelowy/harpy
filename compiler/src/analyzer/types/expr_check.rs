use std::collections::HashSet;

use crate::{
    analyzer::{
        analyzer::Analyzer,
        types::types::{ResolvedType, TypeId},
    }, err::Kind, get_ty, lexer::{
        span::Span,
        tokens::{Ident, Lit, Literal},
    }, parser::{
        Node, expr::{
            expr_defs::{CallExpr, ClosureExpr, Expr, IfExpr, LoopExpr, SwitchExpr},
            ops::{AssignOp, InfixOp, PrefixOp},
        }, node::NodeId
    }
};

impl Analyzer {
    fn check_literal(&mut self, lit: &Literal) -> TypeId {
        let lit = lit.value();
        match lit {
            Lit::LitVoid => TypeId::void(),
            Lit::LitInt(_) => TypeId::int(),
            Lit::LitStr(_) => TypeId::str(),
            Lit::LitBool(_) => TypeId::bool(),
            Lit::LitFloat(_) => TypeId::float(),
        }
    }

    fn check_assign(&mut self, target: &Node<Expr>, op: &AssignOp, value: &Node<Expr>) -> TypeId {
        let target_ty = self.check_expr(target);
        let value_ty = self.check_expr(value);

        if !self.is_lvalue(target) {
            self.report_error(target.span, Kind::InvalidLValue);
        }

        let mut final_target = target_ty;

        if !matches!(op, AssignOp::Eq) && !self.is_compatible(target_ty, value_ty) {
            let derefed_target = self.auto_deref(target.id, target_ty);
            
            if self.is_compatible(derefed_target, value_ty) {
                final_target = derefed_target;
                
            }
        }

        let unified_target = self.check_binding(final_target, value_ty, Some(value.id), value.span);

        if target_ty == TypeId::unknown() {
            if let Expr::Ident(_) = &target.inner {
                self.infer_type(target.id, unified_target);
            }
        }

        match op {
            AssignOp::Eq => (),
            AssignOp::Add | AssignOp::Sub | AssignOp::Mul | AssignOp::Div | AssignOp::Mod => {
                let target_base = self.auto_deref(target.id, unified_target);
                if target_base != TypeId::int() && target_base != TypeId::float() {
                    self.report_error(
                        value.span,
                        Kind::InvalidArithmetic(target_base, value_ty),
                    );
                }
            }
        }

        TypeId::void()
    }

    fn check_infix(&mut self, left: &Node<Expr>, op: &InfixOp, right: &Node<Expr>) -> TypeId {
        let left_ty = self.check_expr(left);
        let right_ty = self.check_expr(right);

        if left_ty == TypeId::unknown() || right_ty == TypeId::unknown() {
            return TypeId::unknown();
        }

        let left_base = self.auto_deref(left.id, left_ty);
        self.ensure_compatible(left_base, right_ty, Some(right.id), right.span);

        match op {
            InfixOp::Add | InfixOp::Sub | InfixOp::Mul | InfixOp::Div | InfixOp::Mod => {
                if left_base != TypeId::int() && left_base != TypeId::float() {
                    self.report_error(
                        right.span,
                        Kind::InvalidArithmetic(left_base, right_ty),
                    );
                    return TypeId::unknown();
                }
                left_base
            }
            InfixOp::Gt | InfixOp::GtEq | InfixOp::Lt | InfixOp::LtEq => {
                if left_base != TypeId::int() && left_base != TypeId::float() {
                    self.report_error(
                        right.span,
                        Kind::InvalidRelational(left_base, right_ty),
                    );
                    return TypeId::unknown();
                }
                TypeId::bool()
            }

            InfixOp::And | InfixOp::Or => {
                self.ensure_compatible(TypeId::bool(), left_ty, Some(right.id), right.span);
                TypeId::bool()
            }

            InfixOp::Eq | InfixOp::Neq => TypeId::bool(),
        }
    }

    fn check_prefix(&mut self, op: &PrefixOp, right: &Node<Expr>) -> TypeId {
        let right_ty = self.check_expr(right);
        if right_ty == TypeId::unknown() {
            return TypeId::unknown();
        }

        let right_base = self.auto_deref(right.id, right_ty);

        match op {
            PrefixOp::Minus | PrefixOp::Plus => {
                if right_base == TypeId::int() || right_base == TypeId::float() {
                    right_base
                } else {
                    self.report_error(right.span,Kind::InvalidPrefix(right_base));
                    TypeId::unknown()
                }
            }
            PrefixOp::Neg => {
                self.ensure_compatible(TypeId::bool(), right_ty, Some(right.id), right.span);
                TypeId::bool()
            }
        }
    }

    fn check_member_access(&mut self, obj: &Node<Expr>, field: &Ident) -> TypeId {
        let obj_ty = self.check_expr(obj);
        let base_ty = self.auto_deref(obj.id, obj_ty);

        if base_ty == TypeId::unknown() {
            return TypeId::unknown();
        }

        let resolved = base_ty.get(self);
        if let ResolvedType::Struct(struct_id) = resolved {
            let layout = struct_id.get(self);
            if let Some(f) = layout.fields.iter().find(|f| f.name == *field.value()) {
                return f.ty;
            } else {
                self.report_error(
                    field.span(),
                    Kind::UnknownField(*struct_id, field.value().clone()),
                );
            }
        } else {
            self.report_error(obj.span, Kind::NotAStruct(base_ty));
        }

        TypeId::unknown()
    }

    fn check_index(&mut self, array: &Node<Expr>, index: &Node<Expr>) -> TypeId {
        let arr_ty = self.check_expr(array);
        let idx_ty = self.check_expr(index);

        let base_ty = self.auto_deref(array.id, arr_ty);
        if base_ty == TypeId::unknown() {
            return TypeId::unknown();
        }

        let inner_ty = match base_ty.get(self) {
            ResolvedType::Array(inner, _) => *inner,
            _ => {
                self.report_error(array.span, Kind::NotAnArray(base_ty));
                return TypeId::unknown()
            }
        };
    
        let resolved_idx = idx_ty.get(self);

        match resolved_idx {
            ResolvedType::Range(inner) => {
                let slice_ty = ResolvedType::Array(inner_ty, None);
                self.ensure_compatible(TypeId::int(), *inner, Some(index.id), index.span);
                self.db.type_table.register(slice_ty)
            }
            _ => {
                self.ensure_compatible(TypeId::int(), idx_ty, Some(index.id), index.span);
                inner_ty
            }
        }

    }

    fn check_box(&mut self, inner: &Node<Expr>, is_mut: bool) -> TypeId {
        let inner_ty = self.check_expr(inner);
        let ty = inner_ty.get(self);

        match ty {
            ResolvedType::Boxed(_, _) | ResolvedType::Ref(_, _) => {
                self.report_error(inner.span, Kind::RecursiveBox);
            }
            _ => ()
        }

        let boxed_ty = ResolvedType::Boxed(inner_ty, is_mut);
        self.db.type_table.register(boxed_ty)
    }

    fn check_ref(&mut self, inner: &Node<Expr>, is_mut: bool) -> TypeId {
        let inner_ty = self.check_expr(inner);
        if !self.is_lvalue(inner) {
            self.report_error(inner.span, Kind::InvalidLValue);
        }

        let ref_ty = ResolvedType::Ref(inner_ty, is_mut);
        self.db.type_table.register(ref_ty)
    }

    fn check_array_init(&mut self, elements: &[Node<Expr>]) -> TypeId {
        let size = elements.len();
        if size == 0 {
            let ty = ResolvedType::Array(TypeId::unknown(), Some(0));
            return self.db.type_table.register(ty);
        }

        let mut base_ty = self.check_expr(&elements[0]);
        for el in elements.iter().skip(1) {
            let el_ty = self.check_expr(el);
            if let Some(unified) = self.unify(base_ty, el_ty) {
                base_ty = unified;
            }
            self.ensure_compatible(base_ty, el_ty, Some(el.id), el.span);
        }

        let ty = ResolvedType::Array(base_ty, Some(size as u64));
        self.db.type_table.register(ty)
    }

    fn check_struct_init(
        &mut self,
        fields: &[(Ident, Node<Expr>)],
        expr_id: NodeId,
        span: Span,
    ) -> TypeId {
        let struct_id = self.db.struct_table.get_resolution(expr_id);
        let layout = struct_id.get(self).clone();
        let mut initialized_fields = HashSet::new();
        for (name, expr) in fields {
            let expr_ty = self.check_expr(expr);
            initialized_fields.insert(name.value());

            if let Some(exp_field) = layout.fields.iter().find(|f| f.name == *name.value()) {
                self.ensure_compatible(exp_field.ty, expr_ty, Some(expr.id), expr.span);
            } else {
                self.report_error(
                    name.span(),
                    Kind::UnknownField(struct_id, name.value().clone())
                );
            }
        }

        for exp_field in &layout.fields {
            if !initialized_fields.contains(&exp_field.name) {
                self.report_error(
                    span,
                    Kind::MissingField(struct_id, exp_field.name.clone())
                );
            }
        }

        self.db.type_table.register(ResolvedType::Struct(struct_id))
    }

    fn check_range(&mut self, start: &Option<Box<Node<Expr>>>, end: &Option<Box<Node<Expr>>>) -> TypeId {
        if let Some(s) = start {
            let s_ty = self.check_expr(s);
            self.ensure_compatible(TypeId::int(), s_ty, Some(s.id), s.span);
        }
        
        if let Some(e) = end {
            let e_ty = self.check_expr(e);
            self.ensure_compatible(TypeId::int(), e_ty, Some(e.id), e.span);
        }

        let ty = ResolvedType::Range(TypeId::int());
        self.db.type_table.register(ty)
    }

    fn check_call(&mut self, call: &Node<CallExpr>) -> TypeId {
        let callee_ty = self.check_expr(&call.callee);
        let args = &call.args;

        if callee_ty == TypeId::unknown() {
            return TypeId::unknown();
        }

        let resolved_callee = callee_ty.get(self).clone();

        if let ResolvedType::Function {
            args: t_args,
            return_type,
        } = resolved_callee
        {
            if t_args.len() != args.len() {
                self.report_error(
                    call.span,
                    Kind::ArgumentCountMismatch(t_args.len(), args.len())
                );
            }

            for (i, arg) in args.iter().enumerate() {
                let arg_ty = self.check_expr(arg);

                if let Some(expected_param_ty) = t_args.get(i) {
                    self.ensure_compatible(*expected_param_ty, arg_ty, Some(arg.id), arg.span);
                }
            }

            return return_type;
        }

        self.report_error(call.callee.span, Kind::NotCallable(callee_ty).into());
        TypeId::unknown()
    }

    fn check_closure(&mut self, closure: &Node<ClosureExpr>) -> TypeId {
        let fn_id = self.db.function_table.get_resolution(closure.id);

        self.with_function(fn_id, |analyzer| {
            let body_ty = analyzer.check_block_expr(&closure.block);
            
            let expected_return_ty = get_ty!(analyzer(fn_id.get(analyzer).signature), ResolvedType::Function { return_type, .. } => return_type);

            analyzer.ensure_compatible(*expected_return_ty, body_ty, Some(closure.block.id), closure.block.span);
        });

        fn_id.get(self).signature
    }

    fn check_return(&mut self, expr: &Option<Box<Node<Expr>>>, span: Span) -> TypeId {
        let ret_ty = if let Some(expr) = expr {
            self.check_expr(expr)
        } else {
            TypeId::void()
        };

        let fn_id = self.current_function();
        if !fn_id.is_valid() {
            self.report_error(span, Kind::ReturnOutsideFn);
        }

        let fn_def = fn_id.get(self);
        let expected_return_ty = get_ty!(self(fn_def.signature), ResolvedType::Function { return_type, .. } => return_type);
        self.ensure_compatible(*expected_return_ty, ret_ty, expr.as_ref().map(|e| e.id), span);

        TypeId::never()
    }

    fn check_break(&mut self, expr: &Option<Box<Node<Expr>>>, span: Span) -> TypeId {
        let ty = expr.as_ref().map_or(TypeId::void(), |e| self.check_expr(e));

        if self.loop_context().is_none() {
            self.report_error(span, Kind::BreakOutsideLoop);
            return TypeId::never();
        }

        let expected_opt = self.loop_context().unwrap().expected_break_ty;

        let updated_ty = match expected_opt {
            Some(exp) => {
                self.unify(exp, ty).unwrap_or_else(|| {
                    self.report_error(span, Kind::NotCompatible{expected: exp, got: ty});
                    exp 
                })
            }
            None => ty,
        };

        self.loop_context().unwrap().expected_break_ty = Some(updated_ty);

        TypeId::never()
    }

    fn check_continue(&mut self, span: Span) -> TypeId {
        if self.loop_context().is_none() {
            self.report_error(span, Kind::ContinueOutsideLoop);
        }
        TypeId::never()
    }

    fn check_if(&mut self, if_expr: &Node<IfExpr>) -> TypeId {
        let cond_ty = self.check_expr(&if_expr.expr);
        self.ensure_compatible(TypeId::bool(), cond_ty, Some(if_expr.expr.id), if_expr.span);

        let then_ty = self.check_block_expr(&if_expr.block);

        if let Some(else_branch) = &if_expr.else_block {
            let else_ty = self.check_expr(else_branch);

            if let Some(unified) = self.unify(then_ty, else_ty) {
                return unified;
            } else {
                self.report_error(
                    else_branch.span,
                    Kind::IfNotCompatible{expected: then_ty, got: else_ty}, 
                );
                return TypeId::unknown();
            }
        } else {
            self.ensure_compatible(TypeId::void(), then_ty, Some(if_expr.block.id), if_expr.span);
            return TypeId::void();
        }
    }

    fn check_loop(&mut self, loop_expr: &Node<LoopExpr>) -> TypeId {
        self.with_loop(|analyzer| {
            analyzer.check_block_expr(&loop_expr.block);
        })
    }

    fn check_switch(&mut self, switch: &Node<SwitchExpr>, span: Span) -> TypeId {
        let target_ty = self.check_expr(&switch.expr);
        
        let mut final_return_ty = TypeId::unknown();
        let mut all_branches_never = true;

        for case in &switch.cases {
            let pattern_ty = self.check_expr(&case.0);
            self.ensure_compatible(target_ty, pattern_ty, Some(case.0.id), case.0.span);

            let branch_ty = self.check_block_expr(&case.1);

            if branch_ty != TypeId::never() {
                all_branches_never = false;
            }

            if final_return_ty == TypeId::unknown() {
                final_return_ty = branch_ty;
            } else if let Some(unified) = self.unify(final_return_ty, branch_ty) {
                final_return_ty = unified;
            } else {
                self.report_error(
                    case.1.span,
                    Kind::NotCompatible{expected: final_return_ty, got: branch_ty} 
                );
            }
        }

        if let Some(default_branch) = &switch.default {
            let def_ty = self.check_block_expr(default_branch);
            
            if def_ty != TypeId::never() {
                all_branches_never = false;
            }

            if final_return_ty == TypeId::unknown() {
                final_return_ty = def_ty;
            } else if let Some(unified) = self.unify(final_return_ty, def_ty) {
                final_return_ty = unified;
            } else {
                self.report_error(
                    default_branch.span,
                    Kind::NotCompatible{expected: final_return_ty, got: def_ty}, 
                );
            }
        } else if final_return_ty != TypeId::void() && final_return_ty != TypeId::unknown() {
            self.report_error(span, Kind::MissingDefaultBranch);
        }

        if all_branches_never {
            TypeId::never()
        } else {
            final_return_ty
        }
    }

    fn check_expr_inner(&mut self, expr: &Node<Expr>) -> TypeId {
        match &expr.inner {
            Expr::Literal(l) => self.check_literal(l),
            Expr::Ident(_) => self.get_symbol_type(expr.id),

            Expr::Assign(target, op, value) => self.check_assign(target, op, value),
            Expr::Infix(left, op, right) => self.check_infix(left, op, right),
            Expr::Prefix(op, right) => self.check_prefix(op, right),
            Expr::MemberAccess(obj, field) => self.check_member_access(obj, field),
            Expr::Index(array, index) => self.check_index(array, index),
            Expr::Box(expr, m) => self.check_box(expr, *m),
            Expr::Borrow(expr, m) => self.check_ref(expr, *m),
            Expr::ArrayInit(el) => self.check_array_init(el),
            Expr::StructInit(_, fields) => self.check_struct_init(fields, expr.id, expr.span),
            Expr::Closure(closure) => self.check_closure(closure),
            Expr::Call(call) => self.check_call(call),
            Expr::Return(ret) => self.check_return(ret, expr.span),
            Expr::Break(br) => self.check_break(br, expr.span),
            Expr::Continue => self.check_continue(expr.span),
            Expr::Loop(l) => self.check_loop(l),
            Expr::If(if_expr) => self.check_if(if_expr),
            Expr::Block(b) => self.check_block_expr(b),
            Expr::Implicit => { self.report_error(expr.span, Kind::UnexpectedDot); TypeId::unknown() },
            Expr::Range(start, end) => self.check_range(start, end),
            Expr::Switch(switch) => self.check_switch(switch, expr.span),
            Expr::Clone(expr) => self.check_expr(expr)
        }
    }

    pub(in crate::analyzer::types) fn check_expr(&mut self, expr: &Node<Expr>) -> TypeId {
        let ty = self.check_expr_inner(expr);
        self.db.type_table.cache(expr.id, ty);
        ty
    }
}
