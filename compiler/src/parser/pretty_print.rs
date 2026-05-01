use super::{
    expr::expr_defs::{BlockExpr, Expr},
    stmt::stmts::{Program, Stmt},
};

pub struct AstPrettyPrint {
    indent: usize,
    output: String,
}

impl AstPrettyPrint {
    pub fn new() -> Self {
        Self {
            indent: 0,
            output: String::new(),
        }
    }

    pub fn print(mut self, program: &Program) -> String {
        self.output.push_str("Program\n");
        self.indent += 1;
        for item in &program.items {
            self.visit_stmt(item);
        }
        self.output
    }

    fn pad(&mut self) {
        self.output.push_str(&"  ".repeat(self.indent));
    }

    fn push_line(&mut self, text: &str) {
        self.pad();
        self.output.push_str(text);
        self.output.push('\n');
    }

    pub fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(s) => {
                self.push_line(&format!("LetStmt (name: {})", s.name.value()));
                self.indent += 1;
                if let Some(_t) = &s.ttype {
                    self.push_line("Type:");
                    self.indent += 1;
                    self.push_line(&format!("{:?}", _t.inner));
                    self.indent -= 1;
                }
                if let Some(e) = &s.expr {
                    self.visit_expr(&e.inner);
                }
                self.indent -= 1;
            }
            Stmt::Global(s) => {
                self.push_line(&format!("GlobalStmt (name: {})", s.name.value()));
                self.indent += 1;
                self.visit_expr(&s.expr.inner);
                self.indent -= 1;
            }
            Stmt::For(s) => {
                self.push_line(&format!("ForStmt (var: {})", s.temp_var.value()));
                self.indent += 1;
                self.push_line("Iterator:");
                self.indent += 1;
                self.visit_expr(&s.iter_expr.inner);
                self.indent -= 1;
                self.push_line("Block:");
                self.indent += 1;
                self.visit_block(&s.block.inner);
                self.indent -= 2;
            }
            Stmt::While(s) => {
                self.push_line("WhileStmt");
                self.indent += 1;
                self.push_line("Condition:");
                self.indent += 1;
                self.visit_expr(&s.expr.inner);
                self.indent -= 1;
                self.push_line("Block:");
                self.indent += 1;
                self.visit_block(&s.block.inner);
                self.indent -= 2;
            }
            Stmt::Function(s) => {
                self.push_line(&format!("FunctionDecl (name: {})", s.name.value()));
                let mut args = String::new();
                for arg in &s.args {
                    args.push_str(&format!("{} ", arg.name.value()));
                }
                self.indent += 1;
                self.push_line(&format!("Args: {}", args));
                self.push_line("Block:");
                self.indent += 1;
                self.visit_block(&s.block.inner);
                self.indent -= 2;
            }
            Stmt::Struct(s) => {
                self.push_line(&format!("StructDecl (name: {})", s.name.value()));
                self.indent += 1;
                for field in &s.fields {
                    self.push_line(&format!("Field (name: {})", field.inner.name.value()));
                }
                self.indent -= 1;
            }
            Stmt::Expr(e) => {
                self.visit_expr(&e.inner);
            }
        }
    }

    pub fn visit_block(&mut self, block: &BlockExpr) {
        for expr in &block.stmts {
            self.visit_stmt(&expr.inner);
        }
    }
    pub fn visit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(lit) => self.push_line(&format!("Literal({})", lit.value())),
            Expr::Ident(name) => self.push_line(&format!("Ident({})", name.value())),

            Expr::Prefix(op, right) => {
                self.push_line(&format!("PrefixOp({:?})", op));
                self.indent += 1;
                self.visit_expr(&right.inner);
                self.indent -= 1;
            }
            Expr::Infix(left, op, right) => {
                self.push_line(&format!("InfixOp({:?})", op));
                self.indent += 1;
                self.visit_expr(&left.inner);
                self.visit_expr(&right.inner);
                self.indent -= 1;
            }
            Expr::Assign(left, op, right) => {
                self.push_line(&format!("AssignOp({:?})", op));
                self.indent += 1;
                self.visit_expr(&left.inner);
                self.visit_expr(&right.inner);
                self.indent -= 1;
            }
            Expr::Iter(left, right) => {
                self.push_line("IteratorRange (=>)");
                self.indent += 1;
                self.visit_expr(&left.inner);
                self.visit_expr(&right.inner);
                self.indent -= 1;
            }
            Expr::Call(call) => {
                self.push_line("CallExpr");
                self.indent += 1;
                self.push_line("Callee:");
                self.indent += 1;
                self.visit_expr(&call.inner.callee.inner);
                self.indent -= 1;

                self.push_line("Args:");
                self.indent += 1;
                for arg in &call.inner.args {
                    self.visit_expr(&arg.inner);
                }
                self.indent -= 2;
            }
            Expr::If(if_expr) => {
                self.push_line("IfExpr");
                self.indent += 1;
                self.push_line("Condition:");
                self.indent += 1;
                self.visit_expr(&if_expr.inner.expr.inner);
                self.indent -= 1;
                self.push_line("Then:");
                self.indent += 1;
                self.visit_block(&if_expr.inner.block.inner);
                self.indent -= 1;

                if let Some(else_b) = &if_expr.inner.else_block {
                    self.push_line("Else:");
                    self.indent += 1;
                    self.visit_expr(&else_b.inner);
                    self.indent -= 1;
                }
                self.indent -= 1;
            }
            Expr::Block(b) => {
                self.push_line("BlockExpr");
                self.indent += 1;
                self.visit_block(&b.inner);
                self.indent -= 1;
            }

            Expr::Continue => {
                self.push_line("Continue");
            }
            Expr::Loop(l) => {
                self.push_line("Loop");
                self.indent += 1;
                self.visit_block(&l.inner.block);
                self.indent -= 1;
            }
            Expr::MemberAccess(e, l) => {
                self.push_line("MemberAccess");
                self.indent += 1;
                self.visit_expr(&e.inner);
                self.indent += 1;
                self.push_line(l.value());
                self.indent -= 2;
            }
            Expr::Return(e) => {
                self.push_line("Return");
                if let Some(expr) = e {
                    self.indent += 1;
                    self.visit_expr(&expr.inner);
                    self.indent -= 1;
                }
            }
            Expr::Break(e) => {
                self.push_line("Break");
                if let Some(expr) = e {
                    self.indent += 1;
                    self.visit_expr(&expr.inner);
                    self.indent -= 1;
                }
            }
            Expr::Switch(s) => {
                self.push_line("Switch");
                self.indent += 1;
                for case in &s.inner.cases {
                    self.visit_expr(&case.inner.expr.inner);
                    self.indent += 1;
                    self.visit_block(&case.inner.block.inner);
                    self.indent -= 1;
                }
                self.indent -= 1;
            }
            Expr::Closure(_) => {
                self.push_line("Closure");
            }
            Expr::Borrow(e, m) => {
                self.push_line(&format!("Borrow ({} mutable)", if *m { "" } else { "not" }));
                self.indent += 1;
                self.visit_expr(&e.inner);
                self.indent -= 1;
            }
            Expr::Box(expr) => {
                self.push_line("Box");
                self.indent += 1;
                self.visit_expr(&expr.inner);
                self.indent -= 1;
            }
            Expr::StructInit(n, f) => {
                self.push_line(n.value());
                self.indent += 1;
                for field in f {
                    self.push_line(&format!("Field: {}", field.0.value()));
                    self.indent += 1;
                    self.visit_expr(&field.1.inner);
                    self.indent -= 1;
                }
                self.indent -= 1;
            }
            Expr::ArrayInit(exprs) => {
                self.push_line("ArrayInit");
                self.indent += 1;
                for e in exprs {
                    self.visit_expr(&e.inner);
                }
                self.indent -= 1;
            }
            Expr::Index(a, e) => {
                self.push_line("Index");
                self.indent += 1;
                self.visit_expr(&a.inner);
                self.visit_expr(&e.inner);
                self.indent -= 1;
            }
        }
    }
}
