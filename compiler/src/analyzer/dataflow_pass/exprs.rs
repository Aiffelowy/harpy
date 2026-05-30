use crate::{
    analyzer::dataflow_pass::data_analyzer::{DataflowAnalyzer, VarState},
    err::Kind,
    parser::{
        expr::{
            expr_defs::{BlockExpr, Expr},
            ops::AssignOp,
        },
        Node,
    },
};

impl<'a> DataflowAnalyzer<'a> {
    fn read_expr(&mut self, expr: &Node<Expr>) {
        if let Expr::Ident(_) = &expr.inner {
            let Some(sym_id) = self.db.symbol_table.resolutions.get(&expr.id) else {
                return;
            };
            let state = self.states.get(sym_id).unwrap_or(&VarState::Uninit);

            match state {
                VarState::Uninit => {
                    self.report_error(expr.span, Kind::UseOfUninit);
                }
                VarState::Moved => {
                    self.report_error(expr.span, Kind::UseOfMoved);
                }
                _ => {}
            }
        } else {
            self.check_expr(expr);
        }
    }

    pub fn check_block(&mut self, block: &Node<BlockExpr>) {
        for s in &block.stmts {
            self.check_stmt(s);
        }
        if let Some(t) = &block.tail {
            self.check_expr(t);
        }
    }

    pub fn check_expr(&mut self, expr: &Node<Expr>) {
        match &expr.inner {
            Expr::Clone(expr) => {
                self.read_expr(expr);
            }
            Expr::Ident(_) => {
                let sym_id = self.db.symbol_table.resolutions[&expr.id];
                self.consume_value(sym_id, expr.span);
            }
            Expr::Literal(_) => {}
            Expr::Assign(l, op, r) => {
                self.check_expr(r);
                if let Some(base) = self.get_base_symbol(l) {
                    if matches!(op, AssignOp::Eq) {
                        self.assign_value(base, expr.span);
                        return;
                    }
                }
                self.check_expr(l);
            }
            Expr::Infix(l, _, r) => {
                self.check_expr(l);
                self.check_expr(r);
            }
            Expr::Prefix(_, r) => {
                self.check_expr(r);
            }
            Expr::MemberAccess(base, _) => {
                self.check_expr(base);
            }
            Expr::Index(base, iexpr) => {
                self.check_expr(base);
                self.check_expr(iexpr);
            }
            Expr::Call(c) => {
                self.check_expr(&c.callee);
                for arg in &c.args {
                    self.check_expr(arg);
                }
            }
            Expr::StructInit(_, f) => {
                for (_, e) in f {
                    self.check_expr(e);
                }
            }
            Expr::ArrayInit(es) => {
                for e in es {
                    self.check_expr(e);
                }
            }
            Expr::Return(r) => {
                if let Some(r) = r {
                    self.check_expr(r);
                    self.check_valid_return(r);
                }

                self.has_returned = true;
            }
            Expr::Break(b) => {
                if let Some(b) = b {
                    self.check_expr(b);
                }
            }
            Expr::Block(b) => {
                self.check_block(b);
            }
            Expr::Closure(c) => {
                self.check_block(&c.block);
            }
            Expr::Borrow(expr, m) => {
                self.read_expr(expr);
                if !m {
                    return;
                }
                let b = self.get_base_symbol(expr);
                if let Some(b) = b {
                    if !self.is_mutable(b) {
                        self.report_error(expr.span, Kind::CannotMutBorrowImmutable);
                    }
                }
            }
            Expr::Box(expr, _) => {
                self.check_expr(expr);
            }
            Expr::Range(f, t) => {
                if let Some(f) = f {
                    self.check_expr(f);
                }
                if let Some(t) = t {
                    self.check_expr(t);
                }
            }
            Expr::Continue => {}
            Expr::Implicit => {}

            Expr::If(i) => {
                self.check_expr(&i.expr);

                let (true_states, true_ret) = self.branch_state(|a| a.check_block(&i.block));
                let (false_states, false_ret) = self.branch_state(|a| {
                    if let Some(e) = &i.else_block {
                        a.check_expr(e);
                    }
                });

                self.merge_states(true_states, false_states);

                if true_ret && false_ret {
                    self.has_returned = true;
                }
            }

            Expr::Loop(l) => {
                let (loop_states, _) = self.branch_state(|a| {
                    a.check_block(&l.block);
                    a.check_block(&l.block);
                });
                self.merge_states(self.states.clone(), loop_states);
            }
            Expr::Switch(s) => {
                self.check_expr(&s.expr);

                let mut all_branch_states = Vec::new();

                for case in &s.cases {
                    all_branch_states.push(self.branch_state(|a| {
                        a.check_expr(&case.0);
                        a.check_block(&case.1);
                    }));
                }

                all_branch_states.push(self.branch_state(|a| {
                    if let Some(d) = &s.default {
                        a.check_block(d);
                    }
                }));

                let (mut combined_state, mut combined_ret) = all_branch_states.pop().unwrap();

                for (branch, branch_ret) in all_branch_states {
                    self.states = combined_state;
                    self.merge_states(self.states.clone(), branch);
                    combined_state = self.states.clone();
                    combined_ret = combined_ret && branch_ret;
                }
                if combined_ret {
                    self.has_returned = true;
                }
            }
        }
    }
}
