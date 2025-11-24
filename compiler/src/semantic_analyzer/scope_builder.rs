use crate::{
    aliases::{ScopeRc, SymbolInfoRef, TypeInfoRc},
    err::{HarpyError, HarpyErrorKind},
    extensions::{ScopeRcExt, SymbolInfoRefExt, WeakScopeExt},
    generator::instruction::{LocalAddress},
    lexer::tokens::Ident,
    parser::{node::{Node}, program::Program, types::TypeSpanned},
};

use super::{
    analyze_trait::Analyze,
    analyzer::Analyzer,
    err::SemanticError,
    result::AnalysisResult,
    scope::{Scope, ScopeKind},
    symbol_info::{FunctionInfo, GlobalInfo, SymbolInfo, SymbolInfoKind, VariableInfo},
};

pub struct ScopeBuilder {
    errors: Vec<HarpyError>,
    current_scope: ScopeRc,
    result: AnalysisResult,
}

impl ScopeBuilder {
    pub fn new() -> Self {
        let result = AnalysisResult::new();
        Self {
            errors: vec![],
            current_scope: result.scope_tree.clone(),
            result,
        }
    }

    pub fn push_scope(&mut self, kind: ScopeKind) {
        let new_scope = {
            let mut current_scope = self.current_scope.get_mut();
            let new_scope = ScopeRc::new(
                Scope::new(kind, Some(&self.current_scope), current_scope.depth() + 1).into(),
            );
            current_scope.children.push(new_scope.clone());
            new_scope
        };
        self.current_scope = new_scope;
    }

    pub fn pop_scope(&mut self) {
        let parent = self.current_scope.get().parent.upgrade();
        if let Some(parent) = parent {
            self.current_scope = parent;
        }
    }

    fn report_error(&mut self, error: HarpyError) -> &mut Self {
        self.errors.push(error);
        self
    }

    fn define_symbol(
        &mut self,
        ident: &Node<Ident>,
        ty: TypeInfoRc,
        info: SymbolInfoKind,
    ) -> Option<SymbolInfoRef> {
        let symbol = SymbolInfo::new(
            ty,
            info,
            ident.id(),
            self.current_scope.get().depth(),
            ident.span(),
        );
        let symbol = SymbolInfoRef::new(symbol.into());

        let r = self.current_scope.get_mut().define(ident, symbol.clone());
        if let Err(e) = r {
            self.report_error(e);
            return None;
        }

        self.result.node_info.insert(ident.id(), symbol.clone());
        Some(symbol)
    }

    pub fn define_param(&mut self, ident: &Node<Ident>, ty: TypeInfoRc) {
        let Some(sym) = self.define_symbol(ident, ty.clone(), SymbolInfoKind::Param) else {
            return;
        };

        let Some(func) = self.current_scope.get().get_function_symbol() else {
            return;
        };
        let mut func = func.as_function_mut().unwrap();
        self.result.locals_map.insert(
            ident.id(),
            LocalAddress(func.locals.len().try_into().unwrap()),
        );
        func.locals.push(sym);
        func.params.push(ty);
    }

    pub fn define_var(&mut self, ident: &Node<Ident>, ty: TypeInfoRc) {
        let Some(sym) = self.define_symbol(
            ident,
            ty.clone(),
            SymbolInfoKind::Variable(VariableInfo::new()),
        ) else {
            return;
        };

        let Some(func) = self.current_scope.get().get_function_symbol() else {
            return;
        };
        let mut func = func.as_function_mut().unwrap();
        //FIX!!!
        self.result.locals_map.insert(
            ident.id(),
            LocalAddress(func.locals.len().try_into().unwrap()),
        );
        func.locals.push(sym);
    }

    pub fn define_func(&mut self, ident: &Node<Ident>, ty: TypeInfoRc) {
        let s = self.define_symbol(ident, ty, SymbolInfoKind::Function(FunctionInfo::new()));
        if let Some(s) = s {
            let func_id = self.result.function_table.register(ident, s.clone());
            if ident.value() == "main" {
                self.result.main_id = Some(func_id);
            }
        }
    }

    pub fn register_type(&mut self, ttype: &TypeSpanned) -> TypeInfoRc {
        if !ttype.verify_pointers() {
            self.report_error(HarpyError::new(
                HarpyErrorKind::SemanticError(SemanticError::PointerToRef),
                ttype.span(),
            ));
        }
        self.result.type_table.register(ttype)
    }

    pub fn define_global(&mut self, ident: &Node<Ident>, ty: TypeInfoRc) {
        let sym = self.define_symbol(
            ident,
            ty.clone(),
            SymbolInfoKind::Global(GlobalInfo),
        );
        if let Some(s) = sym {
            self.result.global_table.register(ident, s);
        }
    }

    pub(in crate::semantic_analyzer) fn build_analyzer(
        program: &Program,
    ) -> std::result::Result<Analyzer, Vec<HarpyError>> {
        let mut s = Self::new();
        program.build(&mut s);

        Ok(Analyzer::new(s.result, s.errors))
    }

    pub fn into_analyzer(self) -> Analyzer {
        Analyzer::new(self.result, self.errors)
    }
}
