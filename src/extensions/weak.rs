use std::{cell::RefCell, rc::Weak};

use crate::{aliases::ScopeRc, semantic_analyzer::scope::Scope};

pub trait WeakScopeExt {
    fn upgrade(&self) -> Option<ScopeRc>;
}

impl WeakScopeExt for Option<Weak<RefCell<Scope>>> {
    fn upgrade(&self) -> Option<ScopeRc> {
        self.as_ref().and_then(|p| p.upgrade())
    }
}
