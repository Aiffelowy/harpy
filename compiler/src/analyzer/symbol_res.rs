use std::collections::HashMap;

use crate::{
    analyzer::{
        analyzer::Analyzer,
        err::SymbolResError,
        tables::{
            struct_table::StructLayout,
            symbol_table::{Symbol, SymbolId},
        },
        types::types::{ResolvedType, TypeId},
    },
    attempt, get_ty,
    lexer::tokens::Ident,
    parser::{
        expr::expr_defs::{BlockExpr, Expr},
        stmt::stmts::{ForStmt, FunctionDecl, LetStmt, Program, Stmt, StructDecl, WhileStmt},
        Node,
    },
};

#[derive(Debug, Default)]
pub struct Scope {
    pub locals: HashMap<String, SymbolId>,
    pub types: HashMap<String, TypeId>,
}

#[derive(Debug)]
pub struct Environment {
    scopes: Vec<Scope>,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            scopes: vec![Scope::default()],
        }
    }
}

impl Environment {
    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop().expect("Tried to pop root scope");
    }

    pub fn declare_local(&mut self, name: String, symbol_id: SymbolId) {
        let current_scope = self.scopes.last_mut().expect("No root scope");
        current_scope.locals.insert(name, symbol_id);
    }

    pub fn resolve_local(&self, name: &str) -> Option<SymbolId> {
        for scope in self.scopes.iter().rev() {
            if let Some(id) = scope.locals.get(name) {
                return Some(*id);
            }
        }
        None
    }

    pub fn declare_type(&mut self, name: String, type_id: TypeId) {
        let current_scope = self.scopes.last_mut().expect("No root scope");
        current_scope.types.insert(name, type_id);
    }

    pub fn resolve_local_type(&self, name: &str) -> Option<TypeId> {
        for scope in self.scopes.iter().rev() {
            if let Some(&id) = scope.types.get(name) {
                return Some(id);
            }
        }
        None
    }

    pub fn has_type_in_current_scope(&self, name: &str) -> bool {
        self.scopes
            .last()
            .map(|scope| scope.types.contains_key(name))
            .unwrap_or(false)
    }
}

impl Analyzer {
    fn resolve_local(&mut self, env: &Environment, name: &Ident) -> Option<SymbolId> {
        if let Some(id) = env.resolve_local(name.value()) {
            return Some(id);
        }

        let global_id = attempt!(self.resolve_global_name(name));
        if !global_id.is_valid() {
            return None;
        }

        let global = self.db.global_table.get(global_id);
        Some(global.symbol)
    }

    fn analyze_let(&mut self, env: &mut Environment, decl: &Node<LetStmt>) {
        if let Some(expr) = &decl.expr {
            self.analyze_expr(env, expr);
        }

        let ty = if let Some(ty) = &decl.ttype {
            attempt!(self.resolve_type_with_env(ty, Some(env)))
        } else {
            TypeId(0)
        };

        let symbol = Symbol::from_let(decl, ty);
        let symbol_id = attempt!(self.register_symbol(symbol));
        if !symbol_id.is_valid() {
            return;
        }

        env.declare_local(decl.name.value().clone(), symbol_id);
        self.db.symbol_table.add_resolution(decl.id, symbol_id);
    }

    fn analyze_struct(&mut self, env: &mut Environment, decl: &Node<StructDecl>) {
        let layout = StructLayout::skeleton(decl.name.value().clone(), decl.name.span());
        attempt!(self.register_struct(layout, Some(env)));
        self.register_struct_fields(decl, Some(env));
    }

    fn analyze_for(&mut self, env: &mut Environment, stmt: &Node<ForStmt>) {
        self.analyze_expr(env, &stmt.iter_expr);
        env.push_scope();

        let symbol = Symbol {
            id: None,
            name: stmt.temp_var.value().clone(),
            ty: TypeId(0),
            is_mutable: false,
            declared_at: stmt.temp_var.span(),
        };

        let symbol_id = attempt!(self.register_symbol(symbol));
        if symbol_id.is_valid() {
            env.declare_local(stmt.temp_var.value().clone(), symbol_id);
        }

        self.analyze_block_expr(env, &stmt.block);
        env.pop_scope();
    }

    fn analyze_while(&mut self, env: &mut Environment, stmt: &Node<WhileStmt>) {
        self.analyze_expr(env, &stmt.expr);
        self.analyze_block_expr(env, &stmt.block);
    }

    fn analyze_expr(&mut self, env: &mut Environment, expr: &Node<Expr>) {}

    fn analyze_stmt(&mut self, env: &mut Environment, stmt: &Stmt) {
        match stmt {
            Stmt::Struct(decl) => {
                self.analyze_struct(env, decl);
            }
            Stmt::Function(decl) => {
                self.register_function_decl(decl, Some(env));
                self.analyze_fn(decl);
            }
            Stmt::Let(decl) => {
                self.analyze_let(env, decl);
            }
            Stmt::For(for_stmt) => {
                self.analyze_for(env, for_stmt);
            }
            Stmt::While(while_stmt) => {
                self.analyze_while(env, while_stmt);
            }
            Stmt::Expr(expr) => {
                self.analyze_expr(env, expr);
            }
            Stmt::Global(g) => {
                self.report_error(SymbolResError::GlobalInFn.into(), g.name.span());
            }
        }
    }

    fn analyze_block_expr(&mut self, env: &mut Environment, block: &Node<BlockExpr>) {
        env.push_scope();
        for stmt in &block.stmts {
            self.analyze_stmt(env, stmt);
        }
        env.pop_scope();
    }

    fn analyze_fn(&mut self, decl: &Node<FunctionDecl>) {
        let mut env = Environment::default();
        let fn_id = attempt!(self.resolve_function_name(&decl.name));
        if fn_id.is_valid() {
            let fn_def = self.db.function_table.get(fn_id);
            let args =
                get_ty!(self(fn_def.signature), ResolvedType::Function {args,..} => args.clone());

            for (param, ty) in decl.args.iter().zip(args.iter()) {
                let symbol = Symbol::from_arg(param, *ty);
                let symbol_id = attempt!(self.register_symbol(symbol));
                if !symbol_id.is_valid() {
                    continue;
                }
                env.declare_local(param.name.value().clone(), symbol_id);
            }
        }

        self.analyze_block_expr(&mut env, &decl.block);
    }

    pub fn symbol_resolution_pass(&mut self, ast: &Program) {
        for stmt in &ast.items {
            if let Stmt::Function(decl) = stmt {
                self.analyze_fn(decl);
            }
        }
    }
}
