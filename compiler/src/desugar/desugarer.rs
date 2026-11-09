use crate::{
    lexer::{
        span::Span,
        tokens::{Ident, Lit, Literal},
    },
    parser::{
        expr::{
            expr_defs::{
                RichBlockExpr, RichCallExpr, RichClosureExpr, RichExpr, RichIfExpr, RichLoopExpr,
                SwitchExpr,
            },
            ops::{AssignOp, InfixOp, PrefixOp},
            simple_expr_defs::{BlockExpr, CallExpr, ClosureExpr, Expr, IfExpr, LoopExpr},
        },
        node::NodeId,
        stmt::{
            stmt_structs::{
                ForStmt, FunctionDecl, GlobalStmt, LetStmt, Program, RichFunctionDecl,
                RichGlobalStmt, RichLetStmt, RichProgram, WhileStmt,
            },
            stmts::{RichStmt, Stmt},
        },
        Node,
    },
};

pub struct Desugarer {
    next_id: u32,
}

impl Desugarer {
    #[inline]
    fn next_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        NodeId(id)
    }

    #[inline]
    fn new_node<T>(&mut self, inner: T, span: Span) -> Node<T> {
        Node::new(self.next_id(), span, inner)
    }

    fn desugar_assign_expr(&mut self, l: Node<RichExpr>, op: AssignOp, r: Node<RichExpr>) -> Expr {
        let infix_op = match op {
            AssignOp::Eq => {
                return Expr::Assign(self.desugar_expr(l), self.desugar_expr(r));
            }
            AssignOp::Add => InfixOp::Add,
            AssignOp::Sub => InfixOp::Sub,
            AssignOp::Mul => InfixOp::Mul,
            AssignOp::Div => InfixOp::Div,
            AssignOp::Mod => InfixOp::Mod,
        };

        let rhs_span = r.span();
        let l_read = self.desugar_expr((l).clone());
        let l_target = self.desugar_expr(l);
        let r_val = self.desugar_expr(r);

        let infix = self.new_node(Expr::Infix(l_read, infix_op, r_val), rhs_span);

        Expr::Assign(l_target, infix)
    }

    fn desugar_call(&mut self, call: Node<RichCallExpr>) -> Node<CallExpr> {
        let (call, span) = call.extract();
        let new = CallExpr {
            callee: self.desugar_expr(call.callee),
            args: call
                .args
                .into_iter()
                .map(|arg| self.desugar_expr(arg))
                .collect(),
        };

        self.new_node(new, span)
    }

    fn desugar_loop(&mut self, l: Node<RichLoopExpr>) -> Node<LoopExpr> {
        let (l, span) = l.extract();
        let new = LoopExpr {
            block: self.desugar_block(l.block),
        };
        self.new_node(new, span)
    }

    fn desugar_if(&mut self, i: Node<RichIfExpr>) -> Node<IfExpr> {
        let (i, span) = i.extract();
        let new = IfExpr {
            expr: self.desugar_expr(i.expr),
            block: self.desugar_block(i.block),
            else_block: i.else_block.map(|b| self.desugar_expr(b)),
        };

        self.new_node(new, span)
    }

    fn desugar_switch(&mut self, switch: Node<SwitchExpr>) -> Node<BlockExpr> {
        let (
            SwitchExpr {
                expr,
                cases,
                default,
            },
            span,
        ) = switch.extract();

        let tmp_id = self.next_id();
        let tmp_ident = Ident::new(expr.span(), format!("<switch_{}>", tmp_id.0));

        let desugared_cond = self.desugar_expr(expr);
        let cond_span = desugared_cond.span();
        let let_stmt = LetStmt {
            name: tmp_ident.clone(),
            mutable: false,
            ttype: None,
            expr: Some(desugared_cond),
        };

        let let_node = Stmt::Let(self.new_node(let_stmt, cond_span));
        let let_node = self.new_node(let_node, cond_span);

        let mut current_else: Option<Node<Expr>> = default.map(|d_block| {
            let block_node = self.desugar_block(d_block);
            let b_span = block_node.span();
            self.new_node(Expr::Block(block_node), b_span)
        });

        for case in cases.into_iter().rev() {
            let case_span = case.0.span();
            let case_pat = self.desugar_expr(case.0);
            let case_body = self.desugar_block(case.1);

            let tmp_read = self.new_node(Expr::Ident(tmp_ident.clone()), case_pat.span());
            let eq_check = self.new_node(Expr::Infix(tmp_read, InfixOp::Eq, case_pat), case_span);

            let if_expr = IfExpr {
                expr: eq_check,
                block: case_body,
                else_block: current_else,
            };

            let if_node = self.new_node(if_expr, case_span);
            current_else = Some(self.new_node(Expr::If(if_node), case_span));
        }

        let block = BlockExpr {
            stmts: vec![let_node],
            tail: current_else,
        };

        self.new_node(block, span)
    }

    fn desugar_closure(&mut self, closure: Node<RichClosureExpr>) -> Node<ClosureExpr> {
        let (closure, span) = closure.extract();
        let new = ClosureExpr {
            args: closure.args,
            return_type: closure.return_type,
            block: self.desugar_block(closure.block),
        };

        self.new_node(new, span)
    }

    fn desugar_for(&mut self, f: Node<ForStmt>) -> Stmt {
        let (f, span) = f.extract();
        let ForStmt {
            start,
            end,
            temp_var,
            block,
        } = f;

        let id = self.next_id;
        let curr_ident = Ident::new(span, format!("<for_curr_{}>", id));
        let end_ident = Ident::new(span, format!("<for_end_{}>", id));
        let step_ident = Ident::new(span, format!("<for_step_{}>", id));

        let start_expr = self.desugar_expr(start);
        let start_span = start_expr.span();
        let let_curr = self.new_node(
            LetStmt {
                name: curr_ident.clone(),
                mutable: true,
                ttype: None,
                expr: Some(start_expr),
            },
            start_span,
        );
        let let_curr = self.new_node(Stmt::Let(let_curr), start_span);

        let end_expr = self.desugar_expr(end);
        let end_span = end_expr.span();
        let let_end = self.new_node(
            LetStmt {
                name: end_ident.clone(),
                mutable: false,
                ttype: None,
                expr: Some(end_expr),
            },
            end_span,
        );
        let let_end = self.new_node(Stmt::Let(let_end), end_span);

        let false_lit = self.new_node(Expr::Literal(Literal::new(span, Lit::LitBool(false))), span);
        let let_step = self.new_node(
            LetStmt {
                name: step_ident.clone(),
                mutable: true,
                ttype: None,
                expr: Some(false_lit),
            },
            span,
        );
        let let_step = self.new_node(Stmt::Let(let_step), span);

        let step_cond = self.new_node(Expr::Ident(step_ident.clone()), span);
        let curr_lhs = self.new_node(Expr::Ident(curr_ident.clone()), span);
        let curr_rhs = self.new_node(Expr::Ident(curr_ident.clone()), span);
        let one_lit = self.new_node(Expr::Literal(Literal::new(span, Lit::LitInt(1))), span);
        let add_expr = self.new_node(Expr::Infix(curr_rhs, InfixOp::Add, one_lit), span);
        let inc_assign = self.new_node(Expr::Assign(curr_lhs, add_expr), span);
        let inc_stmt = self.new_node(Stmt::Semi(inc_assign), span);
        let inc_block = self.new_node(
            BlockExpr {
                stmts: vec![inc_stmt],
                tail: None,
            },
            span,
        );
        let step_if = self.new_node(
            IfExpr {
                expr: step_cond,
                block: inc_block,
                else_block: None,
            },
            span,
        );
        let step_if_stmt = self.new_node(Expr::If(step_if), span);
        let step_if_stmt = self.new_node(Stmt::Expr(step_if_stmt), span);

        let step_lhs = self.new_node(Expr::Ident(step_ident), span);
        let true_lit = self.new_node(Expr::Literal(Literal::new(span, Lit::LitBool(true))), span);
        let set_step_true = self.new_node(Expr::Assign(step_lhs, true_lit), span);
        let set_step_true = self.new_node(Stmt::Semi(set_step_true), span);

        let cmp_curr = self.new_node(Expr::Ident(curr_ident.clone()), start_span);
        let cmp_end = self.new_node(Expr::Ident(end_ident), end_span);
        let lt_expr = self.new_node(Expr::Infix(cmp_curr, InfixOp::Lt, cmp_end), span);
        let not_lt = self.new_node(Expr::Prefix(PrefixOp::Neg, lt_expr), span);
        let break_stmt = self.new_node(Expr::Break(None), span);
        let break_stmt = self.new_node(Stmt::Semi(break_stmt), span);
        let break_block = self.new_node(
            BlockExpr {
                stmts: vec![break_stmt],
                tail: None,
            },
            span,
        );
        let guard_if = self.new_node(
            IfExpr {
                expr: not_lt,
                block: break_block,
                else_block: None,
            },
            span,
        );
        let guard_stmt = self.new_node(Expr::If(guard_if), span);
        let guard_stmt = self.new_node(Stmt::Expr(guard_stmt), span);

        let bind_curr = self.new_node(Expr::Ident(curr_ident), span);
        let let_user_var = self.new_node(
            LetStmt {
                name: temp_var,
                mutable: false,
                ttype: None,
                expr: Some(bind_curr),
            },
            span,
        );
        let let_user_var = self.new_node(Stmt::Let(let_user_var), span);

        let user_body = self.desugar_block(block);
        let body_span = user_body.span();
        let user_body_stmt = self.new_node(Expr::Block(user_body), body_span);
        let user_body_stmt = self.new_node(Stmt::Expr(user_body_stmt), body_span);

        let loop_block = self.new_node(
            BlockExpr {
                stmts: vec![
                    step_if_stmt,
                    set_step_true,
                    guard_stmt,
                    let_user_var,
                    user_body_stmt,
                ],
                tail: None,
            },
            span,
        );

        let loop_expr = self.new_node(LoopExpr { block: loop_block }, span);
        let loop_expr = self.new_node(Expr::Loop(loop_expr), span);
        let loop_stmt = self.new_node(Stmt::Expr(loop_expr), span);

        let outer_block = self.new_node(
            BlockExpr {
                stmts: vec![let_curr, let_end, let_step, loop_stmt],
                tail: None,
            },
            span,
        );

        Stmt::Expr(self.new_node(Expr::Block(outer_block), span))
    }

    fn desugar_while(&mut self, w: Node<WhileStmt>) -> Stmt {
        let (WhileStmt { expr, block }, span) = w.extract();

        let cond = self.desugar_expr(expr);
        let cond_span = cond.span();
        let not_cond = self.new_node(Expr::Prefix(PrefixOp::Neg, cond), cond_span);

        let break_expr = self.new_node(Expr::Break(None), cond_span);
        let break_stmt = self.new_node(Stmt::Semi(break_expr), cond_span);
        let break_block = self.new_node(
            BlockExpr {
                stmts: vec![break_stmt],
                tail: None,
            },
            cond_span,
        );

        let guard_if = self.new_node(
            IfExpr {
                expr: not_cond,
                block: break_block,
                else_block: None,
            },
            cond_span,
        );

        let guard_stmt = Stmt::Expr(self.new_node(Expr::If(guard_if), cond_span));
        let guard_stmt = self.new_node(guard_stmt, cond_span);

        let mut desugared_body = self.desugar_block(block);
        desugared_body.inner_mut().stmts.insert(0, guard_stmt);

        let loop_node = self.new_node(
            LoopExpr {
                block: desugared_body,
            },
            span,
        );
        Stmt::Expr(self.new_node(Expr::Loop(loop_node), span))
    }

    fn desugar_let(&mut self, l: Node<RichLetStmt>) -> Stmt {
        let (l, span) = l.extract();
        let new = LetStmt {
            name: l.name,
            mutable: l.mutable,
            ttype: l.ttype,
            expr: l.expr.map(|e| self.desugar_expr(e)),
        };

        Stmt::Let(self.new_node(new, span))
    }

    fn desugar_expr(&mut self, expr: Node<RichExpr>) -> Node<Expr> {
        let (expr, span) = expr.extract();
        let new = match expr {
            RichExpr::Infix(l, op, r) => {
                Expr::Infix(self.desugar_expr(l), op, self.desugar_expr(r))
            }

            RichExpr::Prefix(op, r) => Expr::Prefix(op, self.desugar_expr(r)),
            RichExpr::Assign(l, op, r) => self.desugar_assign_expr(l, op, r),
            RichExpr::Literal(l) => Expr::Literal(l),
            RichExpr::Ident(i) => Expr::Ident(i),
            RichExpr::Call(c) => Expr::Call(self.desugar_call(c)),
            RichExpr::Loop(l) => Expr::Loop(self.desugar_loop(l)),
            RichExpr::Return(e) => Expr::Return(e.map(|expr| self.desugar_expr(expr))),
            RichExpr::Break(e) => Expr::Break(e.map(|expr| self.desugar_expr(expr))),
            RichExpr::Continue => Expr::Continue,
            RichExpr::If(i) => Expr::If(self.desugar_if(i)),
            RichExpr::Switch(s) => Expr::Block(self.desugar_switch(s)),
            RichExpr::Block(b) => Expr::Block(self.desugar_block(b)),
            RichExpr::Closure(c) => Expr::Closure(self.desugar_closure(c)),
            RichExpr::Borrow(e, m) => Expr::Borrow(self.desugar_expr(e), m),
            RichExpr::Box(e, m) => Expr::Box(self.desugar_expr(e), m),
            RichExpr::Implicit => Expr::Implicit,
            RichExpr::Range(l, r) => Expr::Range(
                l.map(|expr| self.desugar_expr(expr)),
                r.map(|expr| self.desugar_expr(expr)),
            ),
            RichExpr::MemberAccess(e, i) => Expr::MemberAccess(self.desugar_expr(e), i),
            RichExpr::StructInit(i, v) => {
                let mut nv = Vec::with_capacity(v.len());
                for (i, e) in v {
                    nv.push((i, self.desugar_expr(e)));
                }
                Expr::StructInit(i, nv)
            }
            RichExpr::ArrayInit(v) => {
                let mut nv = Vec::with_capacity(v.len());
                for e in v {
                    nv.push(self.desugar_expr(e));
                }
                Expr::ArrayInit(nv)
            }
            RichExpr::Clone(e) => Expr::Clone(self.desugar_expr(e)),
            RichExpr::Index(l, r) => Expr::Index(self.desugar_expr(l), self.desugar_expr(r)),
        };

        self.new_node(new, span)
    }

    fn desugar_stmt(&mut self, stmt: Node<RichStmt>) -> Node<Stmt> {
        let (stmt, span) = stmt.extract();
        let new = match stmt {
            RichStmt::For(f) => self.desugar_for(f),
            RichStmt::While(w) => self.desugar_while(w),
            RichStmt::Let(l) => self.desugar_let(l),
            RichStmt::Semi(e) => Stmt::Semi(self.desugar_expr(e)),
            RichStmt::Expr(e) => Stmt::Expr(self.desugar_expr(e)),
        };

        self.new_node(new, span)
    }

    fn desugar_block(&mut self, block: Node<RichBlockExpr>) -> Node<BlockExpr> {
        let (block, span) = block.extract();
        let new = BlockExpr {
            stmts: block
                .stmts
                .into_iter()
                .map(|stmt| self.desugar_stmt(stmt))
                .collect(),
            tail: block.tail.map(|tail| self.desugar_expr(tail)),
        };

        self.new_node(new, span)
    }

    fn desugar_global(&mut self, global: Node<RichGlobalStmt>) -> Node<GlobalStmt> {
        let (global, span) = global.extract();

        let new = GlobalStmt {
            name: global.name,
            mutable: global.mutable,
            ttype: global.ttype,
            expr: self.desugar_expr(global.expr),
        };

        self.new_node(new, span)
    }

    fn desugar_fn_decl(&mut self, decl: Node<RichFunctionDecl>) -> Node<FunctionDecl> {
        let (decl, span) = decl.extract();
        let new = FunctionDecl {
            name: decl.name,
            args: decl.args,
            return_type: decl.return_type,
            block: self.desugar_block(decl.block),
        };

        self.new_node(new, span)
    }

    pub fn desugar(ast: RichProgram) -> Program {
        let mut desugarer = Self { next_id: 0 };

        Program {
            structs: ast.structs,
            globals: ast
                .globals
                .into_iter()
                .map(|g| desugarer.desugar_global(g))
                .collect(),
            functions: ast
                .functions
                .into_iter()
                .map(|f| desugarer.desugar_fn_decl(f))
                .collect(),
        }
    }
}
