use std::cell::{Ref, RefMut};

use crate::{aliases::ScopeRc, semantic_analyzer::scope::Scope};

pub trait ScopeRcExt {
    fn get(&self) -> Ref<Scope>;
    fn get_mut(&self) -> RefMut<Scope>;
}

impl ScopeRcExt for ScopeRc {
    fn get(&self) -> Ref<Scope> {
        (**self).borrow()
    }

    fn get_mut(&self) -> RefMut<Scope> {
        (**self).borrow_mut()
    }
}
