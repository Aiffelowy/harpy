use std::collections::HashMap;

use crate::{
    aliases::{NodeInfo, ScopeRc, SymbolInfoRef},
    err::HarpyError,
    lexer::tokens::Ident,
    parser::{node::Node, program::Program},
};

use super::{
    analyze_trait::Analyze,
    analyzer::Analyzer,
    scope::{Scope, ScopeKind},
    symbol_info::{FunctionInfo, SymbolInfo, SymbolInfoKind, VariableInfo},
};

pub struct ScopeBuilder {
    errors: Vec<HarpyError>,
    scopes: ScopeRc,
    current_scope: ScopeRc,
    node_info: NodeInfo,
}

impl ScopeBuilder {
    pub fn new() -> Self {
        let root = Scope::new(super::scope::ScopeKind::Global, None);
        let root = ScopeRc::new(root.into());
        Self {
            errors: vec![],
            current_scope: root.clone(),
            scopes: root,
            node_info: HashMap::new(),
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

    fn report_error(&mut self, error: HarpyError) -> &mut Self {
        self.errors.push(error);
        self
    }

    fn define_symbol(&mut self, ident: &Node<Ident>, info: SymbolInfoKind) {
        let symbol = SymbolInfo::new(info, ident.id());
        let symbol = SymbolInfoRef::new(symbol.into());

        let r = (*self.current_scope)
            .borrow_mut()
            .define(ident, symbol.clone());
        if let Err(e) = r {
            self.report_error(e);
            return;
        }

        self.node_info.insert(ident.id(), symbol);
    }

    pub fn define_var(&mut self, ident: &Node<Ident>, info: VariableInfo) {
        self.define_symbol(ident, SymbolInfoKind::Variable(info))
    }

    pub fn define_func(&mut self, ident: &Node<Ident>, info: FunctionInfo) {
        self.define_symbol(ident, SymbolInfoKind::Function(info))
    }

    pub(in crate::semantic_analyzer) fn build_analyzer(
        program: &Program,
    ) -> std::result::Result<Analyzer, Vec<HarpyError>> {
        let mut s = Self::new();
        program.build(&mut s);

        if !s.errors.is_empty() {
            return Err(s.errors);
        }

        Ok(Analyzer::new(s.scopes, s.node_info))
    }
}
