use crate::{
    analyzer::{
        analyzer::Analyzer,
        err::TypeCheckError,
        types::types::{ResolvedType, TypeId},
    },
    attempt, get_ty,
    parser::{
        expr::expr_defs::{BlockExpr, Expr},
        stmt::stmts::{ForStmt, FunctionDecl, GlobalStmt, LetStmt, Program, Stmt, WhileStmt},
        Node,
    },
};

impl Analyzer {
    pub(in crate::analyzer::types) fn check_block_expr(
        &mut self,
        block: &Node<BlockExpr>,
    ) -> TypeId {
        let mut is_never = false;
        let mut return_type = TypeId::void();

        for stmt in &block.stmts {
            if self.check_stmt(stmt) == TypeId::never() {
                is_never = true;
            }
        }

        if let Some(tail) = &block.tail {
            return_type = self.check_expr(tail);
        }

        if is_never {
            TypeId::never()
        } else {
            return_type
        }
    }

    fn check_for_stmt(&mut self, stmt: &Node<ForStmt>) -> TypeId {
        let iter_ty = self.check_expr(&stmt.iter_expr);

        if let Expr::Range(start, _) = &stmt.iter_expr.inner {
            if start.is_none() {
                self.report_error(TypeCheckError::MissingLoopStart.into(), stmt.iter_expr.span);
            }
        }
        let base_iter_ty = self.auto_deref(stmt.iter_expr.id, iter_ty);
        let yield_ty = attempt!(self.get_iterable_yield_type(base_iter_ty, stmt.iter_expr.span));
        self.infer_type(stmt.id, yield_ty);
        self.with_loop(|analyzer| {
            analyzer.check_block_expr(&stmt.block);
        });
        TypeId::void()
    }

    fn check_let_stmt(&mut self, stmt: &Node<LetStmt>) -> TypeId {
        let expr_ty = stmt
            .expr
            .as_ref()
            .map_or(TypeId::unknown(), |e| self.check_expr(e));
        let var_ty = self.get_symbol_type(stmt.id);

        let span = stmt
            .expr
            .as_ref()
            .map_or_else(|| stmt.name.span(), |e| e.span);
        let value_id = stmt.expr.as_ref().map(|e| e.id);

        let unified = self.check_binding(var_ty, expr_ty, value_id, span);

        if var_ty != unified {
            self.infer_type(stmt.id, unified);
        }

        TypeId::void()
    }

    fn check_global_stmt(&mut self, stmt: &Node<GlobalStmt>) -> TypeId {
        let expr_ty = self.check_expr(&stmt.expr);
        let var_ty = self.get_symbol_type(stmt.id);

        let unified = self.check_binding(var_ty, expr_ty, Some(stmt.expr.id), stmt.expr.span);
        if var_ty != unified {
            self.infer_type(stmt.id, unified);
        }

        TypeId::void()
    }

    fn check_while_stmt(&mut self, stmt: &Node<WhileStmt>) -> TypeId {
        let cond_ty = self.check_expr(&stmt.expr);
        self.ensure_compatible(TypeId::bool(), cond_ty, Some(stmt.expr.id), stmt.expr.span);
        self.with_loop(|analyzer| {
            analyzer.check_block_expr(&stmt.block);
        });
        TypeId::void()
    }

    fn check_fn(&mut self, decl: &Node<FunctionDecl>) -> TypeId {
        let fn_id = self.db.function_table.get_resolution(decl.id);

        self.with_function(fn_id, |analyzer| {
            let body_ty = analyzer.check_block_expr(&decl.block);
            let exp_return_ty = get_ty!(analyzer(fn_id.get(analyzer).signature), ResolvedType::Function { return_type, .. } => return_type);

            analyzer.ensure_compatible(*exp_return_ty, body_ty, Some(decl.block.id), decl.block.span);
        });

        TypeId::void()
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> TypeId {
        match stmt {
            Stmt::Semi(expr) => {
                let ty = self.check_expr(expr);
                if ty == TypeId::never() {
                    ty
                } else {
                    TypeId::void()
                }
            }
            Stmt::Expr(expr) => self.check_expr(expr),
            Stmt::Struct(_) => TypeId::void(),
            Stmt::Function(decl) => self.check_fn(decl),
            Stmt::For(for_stmt) => self.check_for_stmt(for_stmt),
            Stmt::Let(let_stmt) => self.check_let_stmt(let_stmt),
            Stmt::Global(global) => self.check_global_stmt(global),
            Stmt::While(while_stmt) => self.check_while_stmt(while_stmt),
        }
    }

    pub(in crate::analyzer) fn check_types(&mut self, ast: &Program) {
        for stmt in &ast.items {
            self.check_stmt(stmt);
        }
    }
}
