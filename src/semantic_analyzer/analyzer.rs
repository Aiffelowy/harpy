use crate::aliases::{Result, SymbolInfoRef, TypeInfoRc};
use crate::err::HarpyErrorKind;
use crate::extensions::{ScopeRcExt, WeakScopeExt};
use crate::lexer::span::Span;
use crate::lexer::tokens::Lit;
use crate::parser::expr::Expr;
use crate::parser::node::Node;
use crate::parser::program::Program;
use crate::parser::types::Type;
use crate::{aliases::ScopeRc, err::HarpyError, lexer::tokens::Ident};

use super::analyze_trait::Analyze;
use super::err::SemanticError;
use super::resolvers::expr_resolver::ExprResolver;
use super::result::AnalysisResult;
use super::scope::ScopeKind;
use super::scope_builder::ScopeBuilder;
use super::symbol_info::{ExprInfo, SymbolInfo, SymbolInfoKind};

#[derive(Debug)]
pub struct Analyzer {
    errors: Vec<HarpyError>,
    #[allow(dead_code)]
    current_scope: ScopeRc,
    result: AnalysisResult,
}

impl Analyzer {
    pub fn new(result: AnalysisResult) -> Self {
        Self {
            errors: vec![],
            current_scope: result.scope_tree.clone(),
            result,
        }
    }

    pub fn enter_scope(&mut self) {
        let current = self.current_scope.clone();
        if let Some(next) = current.get_mut().next_unvisited_child() {
            self.current_scope = next.clone();
        };
    }

    pub fn exit_scope(&mut self) {
        let parent = self.current_scope.get().parent.upgrade();

        if let Some(parent) = parent {
            self.current_scope = parent
        }
    }

    pub fn report_semantic_error(&mut self, error: SemanticError, span: Span) -> &mut Self {
        self.errors
            .push(HarpyError::new(HarpyErrorKind::SemanticError(error), span));
        self
    }

    pub fn report_error(&mut self, error: HarpyError) {
        self.errors.push(error);
    }

    pub fn get_symbol(&mut self, ident: &Ident) -> Result<SymbolInfoRef> {
        self.current_scope.get().lookup(ident)
    }

    pub fn in_scopekind(&self, kind: ScopeKind) -> bool {
        self.current_scope.get().in_scopekind(kind)
    }

    pub fn get_func_info(&self) -> Option<SymbolInfoRef> {
        self.current_scope.get().get_function_symbol()
    }

    pub fn resolve_expr(&mut self, expr: &Node<Expr>) -> Option<TypeInfoRc> {
        match ExprResolver::resolve_expr(expr, self) {
            Ok(t) => {
                let type_info = self.register_type(&t);
                let info = ExprInfo {
                    ttype: type_info.clone(),
                };
                let info = SymbolInfoKind::Expr(info);
                let info = SymbolInfo::new(info, expr.id());

                self.result
                    .node_info
                    .insert(expr.id(), SymbolInfoRef::new(info.into()));
                Some(type_info)
            }
            Err(e) => {
                self.errors.push(e);
                None
            }
        }
    }

    pub fn register_type(&mut self, ttype: &Type) -> TypeInfoRc {
        self.result.type_table.register(ttype)
    }

    pub fn register_constant(&mut self, lit: Lit) {
        self.result.constants.register(lit);
    }

    pub fn main_exists(&self) -> bool {
        self.current_scope.get().main_exists()
    }

    pub fn analyze(program: &Program) -> std::result::Result<AnalysisResult, Vec<HarpyError>> {
        let mut s = ScopeBuilder::build_analyzer(program)?;
        program.analyze_semantics(&mut s);

        if !s.errors.is_empty() {
            return Err(s.errors);
        }

        Ok(s.result)
    }
}

#[macro_export]
macro_rules! get_symbol {
    (($analyzer:ident, $($symbol:tt)+) $name:ident { $($code:tt)* }) => {
        {
            match $analyzer.get_symbol(&$($symbol)+) {
                Err(e) => { $analyzer.report_error(e); }
                Ok(s) => {
                    let mut $name = (*s).borrow_mut();
                    $($code)*
                },
            }
        }
    };
}
