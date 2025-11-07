use crate::aliases::{Result, SymbolInfoRef, TypeInfoRc};
use crate::err::HarpyErrorKind;
use crate::extensions::{ScopeRcExt, WeakScopeExt};
use crate::lexer::span::Span;
use crate::lexer::tokens::Literal;
use crate::parser::expr::expr::CallExpr;
use crate::parser::expr::Expr;
use crate::parser::node::Node;
use crate::parser::program::Program;
use crate::parser::types::{Type, TypeSpanned};
use crate::{aliases::ScopeRc, err::HarpyError, lexer::tokens::Ident};

use super::analyze_trait::Analyze;
use super::err::SemanticError;
use super::resolvers::expr_resolver::{ExprResolver, ResolveMode};
use super::result::AnalysisResult;
use super::scope::{Depth, ScopeKind};
use super::scope_builder::ScopeBuilder;
use super::symbol_info::{BorrowInfo, LiteralInfo, SymbolInfo, SymbolInfoKind};

#[macro_export]
macro_rules! get_symbol {
    (($analyzer:ident, $($symbol:tt)+) $name:ident { $($code:tt)* }) => {
        {
            match $analyzer.get_symbol(&$($symbol)+) {
                Err(e) => { $analyzer.report_error(e); }
                Ok(s) => {
                    let $name = (*s).borrow();
                    $($code)*
                },
            }
        }
    };
}

#[macro_export]
macro_rules! get_symbol_mut {
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

#[derive(Debug)]
pub struct Analyzer {
    errors: Vec<HarpyError>,
    #[allow(dead_code)]
    current_scope: ScopeRc,
    result: AnalysisResult,
}

impl Analyzer {
    pub(in crate::semantic_analyzer) fn new(
        result: AnalysisResult,
        errors: Vec<HarpyError>,
    ) -> Self {
        Self {
            errors,
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
            let r = self.current_scope.get().resolve_borrows();
            match r {
                Ok(()) => (),
                Err(e) => self.report_error(e),
            }
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

    fn res_expr(&mut self, expr: &Node<Expr>, mode: ResolveMode) -> Option<TypeInfoRc> {
        match ExprResolver::resolve_expr(expr, self, mode) {
            Ok(t) => {
                let type_info = self.register_type(&TypeSpanned {
                    ty: t,
                    span: expr.span(),
                });
                let info = SymbolInfo::new(
                    type_info.clone(),
                    SymbolInfoKind::Expr,
                    expr.id(),
                    self.current_scope.get().depth(),
                );

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

    pub fn resolve_expr(&mut self, expr: &Node<Expr>) -> Option<TypeInfoRc> {
        self.res_expr(expr, ResolveMode::Read)
    }

    pub fn resolve_expr_write(&mut self, expr: &Node<Expr>) -> Option<TypeInfoRc> {
        self.res_expr(expr, ResolveMode::Write)
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

    fn register_type_unchecked(&mut self, ttype: &Type) -> TypeInfoRc {
        self.result.type_table.register(ttype)
    }

    pub fn register_constant(&mut self, lit: &Node<Literal>, ty: &Type) {
        let ttype = self.register_type_unchecked(ty);
        let const_idx = self.result.constants.register(lit.value().clone(), &ttype);

        let info = LiteralInfo { const_idx };
        let info = SymbolInfoKind::Literal(info);
        let info = SymbolInfo::new(ttype, info, lit.id(), self.current_scope.get().depth());
        let info = SymbolInfoRef::new(info.into());
        self.result.node_info.insert(lit.id(), info);
    }

    pub fn register_call(&mut self, ident: &Ident, call_expr: &Node<CallExpr>) {
        match self.result.function_table.register_call(ident, call_expr) {
            Some(()) => (),
            None => self.report_error(HarpyError::new(
                HarpyErrorKind::SemanticError(SemanticError::MissingSymbol(ident.clone())),
                ident.span(),
            )),
        }
    }

    pub(in crate::semantic_analyzer) fn current_depth(&self) -> Depth {
        self.current_scope.get().depth()
    }

    pub(in crate::semantic_analyzer) fn register_borrow(&self, info: BorrowInfo) {
        self.current_scope.get_mut().register_borrow(info);
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

    pub fn check_return_borrow(&mut self, id: &Ident) {
        get_symbol!((self, id) var_info {
        if var_info.scope_depth >= self.current_depth() {
            self.report_error(HarpyError::new(
                HarpyErrorKind::SemanticError(SemanticError::ReturnRefToLocal),
                id.span(),
            ));
        }
        })
    }

    pub fn check_assign_borrow(&mut self, lhs: &Ident, expr: &Node<Expr>) {
        if let Some(i) = expr.lvalue() {
            get_symbol!((self, i) info {
                get_symbol!((self, lhs) lhs_info {
                    if lhs_info.scope_depth < info.scope_depth {
                        self.report_error(HarpyError::new(HarpyErrorKind::SemanticError(SemanticError::LifetimeMismatch), expr.span()));
                    }
                })
            });
        }
    }
}
