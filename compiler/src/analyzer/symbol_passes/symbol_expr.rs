use crate::{
    analyzer::{
        analyzer::Analyzer,
        symbol_passes::symbol_res::Environment,
        tables::{function_table::FunctionDef, symbol_table::Symbol},
    },
    attempt,
    lexer::tokens::Ident,
    parser::{
        expr::expr_defs::{BlockExpr, ClosureExpr, Expr},
        types::type_parsing::FunctionType,
        Node,
    },
};

impl Analyzer {
    pub(in crate::analyzer) fn analyze_block_expr(
        &mut self,
        env: &mut Environment,
        block: &Node<BlockExpr>,
    ) {
        env.push_scope();
        for stmt in &block.stmts {
            self.analyze_stmt(env, stmt);
        }

        if let Some(tail) = &block.tail {
            self.analyze_expr(env, tail);
        }

        env.pop_scope();
    }

    fn analyze_closure(&mut self, env: &mut Environment, closure: &Node<ClosureExpr>) {
        let fn_ty = FunctionType {
            args: closure.args.iter().map(|a| a.ty.clone()).collect(),
            return_type: Box::new(closure.return_type.clone()),
        };
        let ty_id = attempt!(self.resolve_fn_type(&fn_ty, Some(env)));
        if !ty_id.is_valid() {
            return;
        }
        let def = FunctionDef::skeleton(format!("closure:<{}>", ty_id.0), ty_id, closure.span);
        let fn_id = attempt!(self.register_function(def));

        env.push_scope();

        self.with_function(fn_id, |analyzer| {
            if fn_id.is_valid() {
                for arg in &closure.args {
                    let ty_id = attempt!(analyzer.resolve_type_with_env(&arg.ty, Some(env)));
                    let symbol = Symbol::from_arg(arg, ty_id);
                    let symbol_id = attempt!(analyzer.register_symbol(symbol));

                    if symbol_id.is_valid() {
                        env.declare_local(arg.name.value().clone(), symbol_id);
                        analyzer.track_param(symbol_id);
                    }
                }
            }

            analyzer.analyze_block_expr(env, &closure.block);
        });
        env.pop_scope();
    }

    fn analyze_struct_init(
        &mut self,
        env: &mut Environment,
        struct_name: &Ident,
        fields: &[(Ident, Node<Expr>)],
    ) {
        let id = self.resolve_local_struct(Some(env), struct_name);
        if !id.is_valid() {
            return;
        }

        for (_, expr) in fields {
            self.analyze_expr(env, expr);
        }
    }

    pub(in crate::analyzer) fn analyze_expr(&mut self, env: &mut Environment, expr: &Node<Expr>) {
        match &expr.inner {
            Expr::Infix(left, _, right) => {
                self.analyze_expr(env, left);
                self.analyze_expr(env, right);
            }
            Expr::Prefix(_, right) => {
                self.analyze_expr(env, right);
            }
            Expr::Assign(left, _, right) => {
                self.analyze_expr(env, left);
                self.analyze_expr(env, right);
            }
            Expr::Literal(_) => {}
            Expr::Ident(name) => {
                let Some(symbol_id) = self.resolve_local(env, name) else {
                    return;
                };
                self.db.symbol_table.add_resolution(expr.id, symbol_id);
            }
            Expr::Call(c) => {
                self.analyze_expr(env, &c.callee);
                for arg in &c.args {
                    self.analyze_expr(env, arg);
                }
            }
            Expr::Loop(l) => {
                self.analyze_block_expr(env, &l.block);
            }
            Expr::Return(expr) => {
                if let Some(expr) = expr {
                    self.analyze_expr(env, expr);
                }
            }
            Expr::Break(expr) => {
                if let Some(expr) = expr {
                    self.analyze_expr(env, expr);
                }
            }
            Expr::Continue => {}
            Expr::If(if_expr) => {
                self.analyze_expr(env, &if_expr.expr);
                self.analyze_block_expr(env, &if_expr.block);
                if let Some(expr) = &if_expr.else_block {
                    self.analyze_expr(env, expr);
                }
            }
            Expr::Switch(s) => {
                self.analyze_expr(env, &s.expr);
                for case in &s.cases {
                    self.analyze_expr(env, &case.expr);
                    self.analyze_block_expr(env, &case.block);
                }
            }
            Expr::Block(b) => {
                self.analyze_block_expr(env, b);
            }
            Expr::Closure(c) => {
                self.analyze_closure(env, c);
            }
            Expr::Borrow(expr, _) => {
                self.analyze_expr(env, expr);
            }
            Expr::Box(expr) => self.analyze_expr(env, expr),
            Expr::Iter(from, to) => {
                self.analyze_expr(env, from);
                self.analyze_expr(env, to);
            }
            Expr::MemberAccess(expr, _) => {
                self.analyze_expr(env, expr);
            }
            Expr::StructInit(struct_name, fields) => {
                self.analyze_struct_init(env, struct_name, fields);
            }
            Expr::ArrayInit(exprs) => {
                for expr in exprs {
                    self.analyze_expr(env, expr);
                }
            }
            Expr::Index(left, right) => {
                self.analyze_expr(env, left);
                self.analyze_expr(env, right);
            }
        }
    }
}
