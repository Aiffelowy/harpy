use crate::{
    aliases::SymbolInfoRef,
    semantic_analyzer::symbol_info::{FunctionInfo, SymbolInfoKind},
};

pub trait SymbolInfoRefExt {
    fn as_function(&self) -> Option<std::cell::Ref<FunctionInfo>>;
    fn as_function_mut(&self) -> Option<std::cell::RefMut<FunctionInfo>>;
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
}
