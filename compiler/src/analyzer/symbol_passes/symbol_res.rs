use std::collections::HashMap;

use crate::{
    analyzer::{
        analyzer::Analyzer,
        tables::{
            struct_table::{StructId, StructLayout},
            symbol_table::{Symbol, SymbolId, SymbolKind},
        },
        types::types::{ResolvedType, TypeId},
    },
    attempt,
    err::Kind,
    get_ty,
    lexer::tokens::Ident,
    parser::{
        stmt::stmts::{ForStmt, FunctionDecl, LetStmt, Program, Stmt, StructDecl, WhileStmt},
        Node,
    },
};

#[derive(Debug, Default)]
pub struct Scope {
    pub locals: HashMap<String, SymbolId>,
    pub structs: HashMap<String, StructId>,
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

    fn resolve_local(&self, name: &str) -> Option<SymbolId> {
        for scope in self.scopes.iter().rev() {
            if let Some(id) = scope.locals.get(name) {
                return Some(*id);
            }
        }
        None
    }

    pub fn declare_struct(&mut self, name: String, struct_id: StructId) {
        let current_scope = self.scopes.last_mut().expect("No root scope");
        current_scope.structs.insert(name, struct_id);
    }

    fn resolve_local_struct(&self, name: &str) -> Option<StructId> {
        for scope in self.scopes.iter().rev() {
            if let Some(&id) = scope.structs.get(name) {
                return Some(id);
            }
        }
        None
    }

    pub fn has_type_in_current_scope(&self, name: &str) -> bool {
        self.scopes
            .last()
            .map(|scope| scope.structs.contains_key(name))
            .unwrap_or(false)
    }
}

impl Analyzer {
    pub(in crate::analyzer) fn resolve_local(
        &mut self,
        env: &Environment,
        name: &Ident,
    ) -> Option<SymbolId> {
        if let Some(id) = env.resolve_local(name.value()) {
            return Some(id);
        }

        let global_id = attempt!(self.resolve_global_name(name));
        if !global_id.is_valid() {
            return None;
        }

        Some(global_id.get(self).symbol)
    }

    pub(in crate::analyzer) fn resolve_local_struct(
        &mut self,
        env: Option<&Environment>,
        name: &Ident,
    ) -> StructId {
        if let Some(id) = env.and_then(|e| e.resolve_local_struct(name.value())) {
            id
        } else {
            attempt!(self.resolve_struct_name(name))
        }
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
        self.track_local_variable(symbol_id);
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
            name: stmt.temp_var.value().clone(),
            ty: TypeId(0),
            is_mutable: false,
            declared_at: stmt.temp_var.span(),
            kind: SymbolKind::Local,
        };

        let symbol_id = attempt!(self.register_symbol(symbol));
        if symbol_id.is_valid() {
            env.declare_local(stmt.temp_var.value().clone(), symbol_id);
            self.db.symbol_table.add_resolution(stmt.id, symbol_id);
        }

        self.analyze_block_expr(env, &stmt.block);
        env.pop_scope();
    }

    fn analyze_while(&mut self, env: &mut Environment, stmt: &Node<WhileStmt>) {
        self.analyze_expr(env, &stmt.expr);
        self.analyze_block_expr(env, &stmt.block);
    }

    pub(in crate::analyzer) fn analyze_stmt(&mut self, env: &mut Environment, stmt: &Stmt) {
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
                self.report_error(g.name.span(), Kind::GlobalInFn);
            }
            Stmt::Semi(expr) => {
                self.analyze_expr(env, expr);
            }
        }
    }

    fn analyze_fn(&mut self, decl: &Node<FunctionDecl>) {
        let mut env = Environment::default();

        let fn_id = attempt!(self.resolve_function_name(&decl.name));
        self.with_function(fn_id, |analyzer| {
            if fn_id.is_valid() {
                let fn_def = fn_id.get(analyzer);
                let args =
                    get_ty!(analyzer(fn_def.signature), ResolvedType::Function {args,..} => args.clone());

                for (param, ty) in decl.args.iter().zip(args.iter()) {
                    let symbol = Symbol::from_arg(param, *ty);
                    let symbol_id = attempt!(analyzer.register_symbol(symbol));
                    if !symbol_id.is_valid() {
                        continue;
                    }
                    env.declare_local(param.name.value().clone(), symbol_id);
                    analyzer.track_param(symbol_id);
                    analyzer.db.symbol_table.add_resolution(param.id, symbol_id);
                }
            }
            analyzer.analyze_block_expr(&mut env, &decl.block);
        });
    }

    pub fn symbol_resolution_pass(&mut self, ast: &Program) {
        for stmt in &ast.items {
            if let Stmt::Function(decl) = stmt {
                self.analyze_fn(decl);
            }
        }
    }
}
