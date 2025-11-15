use std::{
    cell::{Ref, RefCell},
    rc::Weak,
};

use crate::{aliases::ScopeRc, semantic_analyzer::scope::Scope};

pub trait WeakScopeExt {
    fn upgrade(&self) -> Option<ScopeRc>;
    fn upgrade_then<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(Ref<Scope>) -> R;
}

impl WeakScopeExt for Option<Weak<RefCell<Scope>>> {
    fn upgrade(&self) -> Option<ScopeRc> {
        self.as_ref().and_then(|p| p.upgrade())
    }

    fn upgrade_then<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(Ref<Scope>) -> R,
    {
        Some(f(self.upgrade()?.borrow()))
    }
}
