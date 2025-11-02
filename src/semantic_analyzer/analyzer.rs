use crate::aliases::{Result, SymbolInfoRef};
use crate::parser::expr::Expr;
use crate::parser::types::Type;
use crate::{aliases::ScopeRc, err::HarpyError, lexer::tokens::Ident};

use super::resolvers::expr_resolver::ExprResolver;

pub struct Analyzer {
    errors: Vec<HarpyError>,
    scopes: ScopeRc,
    current_scope: ScopeRc,
}

impl Analyzer {
    pub fn new(scopes: ScopeRc) -> Self {
        Self {
            errors: vec![],
            current_scope: scopes.clone(),
            scopes,
        }
    }

    pub fn enter_scope(&mut self) {
        let current = self.current_scope.clone();
        if let Some(next) = current.borrow_mut().next_unvisited_child() {
            self.current_scope = next.clone();
        };
    }

    pub fn exit_scope(&mut self) {
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

    pub fn get_symbol(&self, ident: &Ident) -> Result<SymbolInfoRef> {
        (*self.current_scope).borrow_mut().lookup(ident)
    }

    pub fn resolve_expr(&self, expr: &Expr) -> Result<Type> {
        ExprResolver::resolve_expr(expr, self)
    }
}
