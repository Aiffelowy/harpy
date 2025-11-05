use std::cell::{Ref, RefMut};

use crate::{
    aliases::SymbolInfoRef,
    semantic_analyzer::symbol_info::{FunctionInfo, SymbolInfo, SymbolInfoKind, VariableInfo},
};

pub trait SymbolInfoRefExt {
    fn as_function(&self) -> Option<std::cell::Ref<FunctionInfo>>;
    fn as_function_mut(&self) -> Option<std::cell::RefMut<FunctionInfo>>;
    fn as_variable(&self) -> Option<std::cell::Ref<VariableInfo>>;
    fn as_variable_mut(&self) -> Option<std::cell::RefMut<VariableInfo>>;
    fn get(&self) -> Ref<SymbolInfo>;
}

impl SymbolInfoRefExt for SymbolInfoRef {
    fn as_function(&self) -> Option<std::cell::Ref<FunctionInfo>> {
        std::cell::Ref::filter_map(self.borrow(), |sym| {
            if let SymbolInfoKind::Function(f) = &sym.kind {
                return Some(f);
            }
            None
        })
        .ok()
    }

    fn as_function_mut(&self) -> Option<std::cell::RefMut<FunctionInfo>> {
        std::cell::RefMut::filter_map(self.borrow_mut(), |sym| {
            if let SymbolInfoKind::Function(ref mut f) = &mut sym.kind {
                return Some(f);
            }
            None
        })
        .ok()
    }

    fn as_variable(&self) -> Option<Ref<VariableInfo>> {
        std::cell::Ref::filter_map(self.borrow(), |sym| {
            if let SymbolInfoKind::Variable(v) = &sym.kind {
                return Some(v);
            }
            None
        })
        .ok()
    }

    fn as_variable_mut(&self) -> Option<RefMut<VariableInfo>> {
        std::cell::RefMut::filter_map(self.borrow_mut(), |sym| {
            if let SymbolInfoKind::Variable(ref mut v) = &mut sym.kind {
                return Some(v);
            }
            None
        })
        .ok()
    }

    fn get(&self) -> Ref<SymbolInfo> {
        (**self).borrow()
    }
}
