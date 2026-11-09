use crate::{
    analyzer::{
        analyzer::Analyzer,
        types::types::{ResolvedType, TypeId},
    },
    err::Kind,
    get_ty,
    parser::{
        expr::simple_expr_defs::BlockExpr,
        stmt::{
            stmt_structs::{FunctionDecl, GlobalStmt, LetStmt, Program, StructDecl},
            stmts::Stmt,
        },
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

    fn check_let_stmt(&mut self, stmt: &Node<LetStmt>) -> TypeId {
        let expr_ty = stmt
            .expr
            .as_ref()
            .map_or(TypeId::unknown(), |e| self.check_expr(e));
        let var_ty = self.get_symbol_type(stmt.id());

        let span = stmt.span().merge(stmt.name.span());

        let value_id = stmt.expr.as_ref().map(|e| e.id());

        let unified = self.check_binding(var_ty, expr_ty, value_id, span);

        if var_ty != unified {
            self.infer_type(stmt.id(), unified);
        }

        TypeId::void()
    }

    fn check_global_stmt(&mut self, stmt: &Node<GlobalStmt>) -> TypeId {
        let expr_ty = self.check_expr(&stmt.expr);
        let var_ty = self.get_symbol_type(stmt.id());

        let unified = self.check_binding(var_ty, expr_ty, Some(stmt.expr.id()), stmt.expr.span());
        if var_ty != unified {
            self.infer_type(stmt.id(), unified);
        }

        TypeId::void()
    }

    fn check_fn(&mut self, decl: &Node<FunctionDecl>) -> TypeId {
        let fn_id = self.db.function_table.get_resolution(decl.id());

        self.with_function(fn_id, |analyzer| {
            let body_ty = analyzer.check_block_expr(&decl.block);
            let exp_return_ty = get_ty!(analyzer(fn_id.get(analyzer).signature), ResolvedType::Function { return_type, .. } => return_type);

            analyzer.ensure_compatible(*exp_return_ty, body_ty, Some(decl.block.id()), decl.block.span());
        });

        TypeId::void()
    }

    fn check_struct(&mut self, decl: &Node<StructDecl>) -> TypeId {
        fn has_ref(analyzer: &Analyzer, ty_id: &TypeId) -> bool {
            let ty = ty_id.get(analyzer);
            match ty {
                ResolvedType::Ref(_, _) => true,
                ResolvedType::Array(id, _) => has_ref(analyzer, id),
                ResolvedType::Boxed(id, _) => has_ref(analyzer, id),
                _ => false,
            }
        }

        for field in &decl.fields {
            let id = self.get_cached_type(field.id());
            if has_ref(self, &id) {
                self.report_error(field.ttype.span(), Kind::RefInStruct);
            }
        }
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
            Stmt::Let(let_stmt) => self.check_let_stmt(let_stmt),
        }
    }

    pub(in crate::analyzer) fn check_types(&mut self, ast: &Program) {
        for s in &ast.structs {
            self.check_struct(s);
        }

        for f in &ast.functions {
            self.check_fn(f);
        }

        for g in &ast.globals {
            self.check_global_stmt(g);
        }
    }
}
