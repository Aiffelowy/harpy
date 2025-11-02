use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::{Rc, Weak},
};

use crate::{
    aliases::{Result, ScopeRc, SymbolInfoRef},
    err::HarpyError,
    lexer::tokens::Ident,
};

use super::{
    err::SemanticError,
    symbol_info::{SymbolInfo, SymbolInfoKind},
};

pub enum ScopeKind {
    Global,
    Function,
    Loop,
    Block,
}

pub struct Scope {
    symbols: HashMap<String, SymbolInfoRef>,
    pub(in crate::semantic_analyzer) parent: Option<Weak<RefCell<Scope>>>,
    pub(in crate::semantic_analyzer) children: Vec<ScopeRc>,
    idx: usize,
    kind: ScopeKind,
}

impl Scope {
    pub(in crate::semantic_analyzer) fn new(kind: ScopeKind, parent: Option<&ScopeRc>) -> Self {
        Self {
            kind,
            parent: parent.map(|p| Rc::downgrade(p)),
            symbols: HashMap::new(),
            children: vec![],
            idx: 0,
        }
    }

    pub(in crate::semantic_analyzer) fn define(
        &mut self,
        ident: &Ident,
        symbol: SymbolInfoKind,
    ) -> Result<()> {
        if self.symbols.contains_key(ident.value()) {
            return HarpyError::semantic(
                SemanticError::DuplicateSymbol(ident.clone()),
                ident.span(),
            );
        }

        self.symbols.insert(
            ident.value().to_owned(),
            SymbolInfoRef::new(SymbolInfo::new(symbol, self.idx).into()),
        );

        self.idx += 1;

        Ok(())
    }

    pub(in crate::semantic_analyzer) fn lookup(&self, ident: &Ident) -> Result<SymbolInfoRef> {
        if let Some(s) = self.symbols.get(ident.value()) {
            (**s).borrow_mut().ref_count += 1;
            return Ok(s.clone());
        }

        if let Some(parent_rc) = self.parent.as_ref().and_then(|p| p.upgrade()) {
            let parent = (*parent_rc).borrow_mut();
            return parent.lookup(ident);
        }

        HarpyError::semantic(SemanticError::MissingSymbol(ident.clone()), ident.span())
    }
}
