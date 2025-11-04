use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use crate::{
    aliases::{Result, ScopeRc, SymbolInfoRef},
    err::HarpyError,
    extensions::{ScopeRcExt, WeakScopeExt},
    lexer::tokens::Ident,
    parser::node::Node,
};

use super::err::SemanticError;

#[derive(Debug, PartialEq)]
pub enum ScopeKind {
    Global,
    Function(String),
    Loop,
    Block,
}

#[derive(Debug)]
pub struct Scope {
    symbols: HashMap<String, SymbolInfoRef>,
    pub(in crate::semantic_analyzer) parent: Option<Weak<RefCell<Scope>>>,
    pub(in crate::semantic_analyzer) children: Vec<ScopeRc>,
    visited: bool,
    kind: ScopeKind,
}

impl Scope {
    pub(in crate::semantic_analyzer) fn new(kind: ScopeKind, parent: Option<&ScopeRc>) -> Self {
        Self {
            kind,
            parent: parent.map(|p| Rc::downgrade(p)),
            symbols: HashMap::new(),
            children: vec![],
            visited: false,
        }
    }

    pub(in crate::semantic_analyzer) fn define(
        &mut self,
        node: &Node<Ident>,
        symbol: SymbolInfoRef,
    ) -> Result<()> {
        if self.symbols.contains_key(node.value()) {
            return HarpyError::semantic(SemanticError::DuplicateSymbol(node.clone()), node.span());
        }

        self.symbols.insert(node.value().to_owned(), symbol);

        Ok(())
    }

    pub(in crate::semantic_analyzer) fn next_unvisited_child(&mut self) -> Option<ScopeRc> {
        for child in &self.children {
            if !child.get().visited {
                child.get_mut().visited = true;
                return Some(child.clone());
            }
        }
        None
    }

    pub(in crate::semantic_analyzer) fn lookup(&self, ident: &Ident) -> Result<SymbolInfoRef> {
        if let Some(s) = self.symbols.get(ident.value()) {
            (**s).borrow_mut().ref_count += 1;
            return Ok(s.clone());
        }

        match self.parent.upgrade().map(|p| p.get().lookup(ident)) {
            Some(s) => s,
            None => HarpyError::semantic(SemanticError::MissingSymbol(ident.clone()), ident.span()),
        }
    }

    pub(in crate::semantic_analyzer) fn in_scopekind(&self, kind: ScopeKind) -> bool {
        if kind == self.kind {
            return true;
        }

        self.parent
            .upgrade()
            .map(|p| p.get().in_scopekind(kind))
            .unwrap_or(false)
    }

    pub(in crate::semantic_analyzer) fn get_function_symbol(&self) -> Option<SymbolInfoRef> {
        if let ScopeKind::Function(name) = &self.kind {
            return self.parent.upgrade_then(|p| p.symbols.get(name).cloned())?;
        }

        self.parent.upgrade_then(|p| p.get_function_symbol())?
    }

    pub(in crate::semantic_analyzer) fn main_exists(&self) -> bool {
        self.symbols.contains_key("main")
    }
}
