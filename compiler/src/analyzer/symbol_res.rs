use std::collections::HashMap;

use crate::{
    analyzer::{analyzer::Analyzer, modules::ModuleId, tables::symbol_table::SymbolId},
    attempt,
    lexer::tokens::Ident,
};

#[derive(Debug, Default)]
pub struct Scope {
    pub locals: HashMap<String, SymbolId>,
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
}

impl Analyzer {
    fn resolve_local(
        &mut self,
        module_id: ModuleId,
        env: &Environment,
        name: &Ident,
    ) -> Option<SymbolId> {
        if let Some(id) = env.resolve_local(name.value()) {
            return Some(id);
        }

        let global_id = attempt!(self, self.resolve_global_name(name));
        let global = self.db.global_table.get(global_id);
        Some(global.symbol)
    }
}
