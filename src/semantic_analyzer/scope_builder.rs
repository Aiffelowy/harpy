use crate::{
    aliases::{ScopeRc, SymbolInfoRef, TypeInfoRc},
    err::HarpyError,
    extensions::{ScopeRcExt, SymbolInfoRefExt, WeakScopeExt},
    lexer::tokens::Ident,
    parser::{node::Node, program::Program, types::Type},
};

use super::{
    analyze_trait::Analyze,
    analyzer::Analyzer,
    result::AnalysisResult,
    scope::{Scope, ScopeKind},
    symbol_info::{FunctionInfo, ParamInfo, SymbolInfo, SymbolInfoKind, VariableInfo},
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
        let new_scope = ScopeRc::new(Scope::new(kind, Some(&self.current_scope)).into());
        self.current_scope
            .get_mut()
            .children
            .push(new_scope.clone());
        self.current_scope = new_scope;
    }

    pub fn pop_scope(&mut self) {
        let parent = self.current_scope.get().parent.upgrade();
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

        let r = self.current_scope.get_mut().define(ident, symbol.clone());
        if let Err(e) = r {
            self.report_error(e);
            return;
        }

        self.result.node_info.insert(ident.id(), symbol);
    }

    pub fn define_param(&mut self, ident: &Node<Ident>, info: ParamInfo) {
        let ttype = info.ttype.clone();
        self.define_symbol(ident, SymbolInfoKind::Param(info));

        let Some(func) = self.current_scope.get().get_function_symbol() else {
            return;
        };
        func.as_function_mut().unwrap().params.push(ttype);
    }

    pub fn define_var(&mut self, ident: &Node<Ident>, info: VariableInfo) {
        let ttype = info.ttype.clone();
        self.define_symbol(ident, SymbolInfoKind::Variable(info));

        let Some(func) = self.current_scope.get().get_function_symbol() else {
            return;
        };
        func.as_function_mut().unwrap().locals.push(ttype);
    }

    pub fn define_func(&mut self, ident: &Node<Ident>, info: FunctionInfo) {
        self.define_symbol(ident, SymbolInfoKind::Function(info))
    }

    pub fn register_type(&mut self, ttype: &Type) -> TypeInfoRc {
        self.result.type_table.register(ttype)
    }

    pub(in crate::semantic_analyzer) fn build_analyzer(
        program: &Program,
    ) -> std::result::Result<Analyzer, Vec<HarpyError>> {
        let mut s = Self::new();
        program.build(&mut s);

        if !s.errors.is_empty() {
            return Err(s.errors);
        }

        Ok(Analyzer::new(s.result))
    }
}
