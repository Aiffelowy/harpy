use crate::aliases::{Result, SymbolInfoRef};
use crate::parser::expr::Expr;
use crate::parser::types::Type;
use crate::{aliases::ScopeRc, err::HarpyError, lexer::tokens::Ident};

use super::resolvers::expr_resolver::ExprResolver;
use super::scope::{Scope, ScopeKind};
use super::symbol_info::{FunctionInfo, SymbolInfoKind, VariableInfo};

pub struct Analyzer {
    errors: Vec<HarpyError>,
    scopes: ScopeRc,
    current_scope: ScopeRc,
}
impl Analyzer {
    pub fn new() -> Self {
        let root = Scope::new(super::scope::ScopeKind::Global, None);
        let root = ScopeRc::new(root.into());
        Self {
            errors: vec![],
            current_scope: root.clone(),
            scopes: root,
        }
    }

    pub fn push_scope(&mut self, kind: ScopeKind) {
        let new_scope = ScopeRc::new(Scope::new(kind, Some(&self.current_scope)).into());
        (*self.current_scope)
            .borrow_mut()
            .children
            .push(new_scope.clone());
        self.current_scope = new_scope;
    }

    pub fn pop_scope(&mut self) {
        let parent = {
            let current = (*self.current_scope).borrow();
            current.parent.as_ref().and_then(|p| p.upgrade())
        };

        if let Some(parent) = parent {
            self.current_scope = parent
        }
    }

    pub fn report_error(&mut self, error: HarpyError) -> &mut Self {
        self.errors.push(error);
        self
    }

    fn define_symbol(&mut self, ident: &Ident, info: SymbolInfoKind) {
        let r = (*self.current_scope).borrow_mut().define(ident, info);
        if let Err(e) = r {
            self.report_error(e);
        }
    }

    pub fn define_var(&mut self, ident: &Ident, info: VariableInfo) {
        self.define_symbol(ident, SymbolInfoKind::Variable(info))
    }

    pub fn define_func(&mut self, ident: &Ident, info: FunctionInfo) {
        self.define_symbol(ident, SymbolInfoKind::Function(info))
    }

    pub fn get_symbol(&self, ident: &Ident) -> Result<SymbolInfoRef> {
        (*self.current_scope).borrow_mut().lookup(ident)
    }

    pub fn resolve_expr(&self, expr: &Expr) -> Result<Type> {
        ExprResolver::resolve_expr(expr, self)
    }
}
