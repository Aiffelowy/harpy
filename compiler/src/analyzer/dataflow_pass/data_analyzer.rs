use std::collections::HashMap;

use crate::{
    analyzer::{
        analyzer::{Analyzer, SemanticDB},
        tables::{
            function_table::FunctionId,
            symbol_table::{SymbolId, SymbolKind},
        },
        types::types::ResolvedType,
    },
    err::{HarpyError, Kind},
    lexer::span::Span,
    parser::{
        expr::expr_defs::Expr,
        stmt::stmts::{Program, Stmt},
        Node,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VarState {
    Uninit,
    Init,
    Moved,
}

pub struct DataflowAnalyzer<'a> {
    pub db: &'a SemanticDB,
    pub states: HashMap<SymbolId, VarState>,
    pub errors: Vec<HarpyError>,
    pub has_returned: bool,
    pub current_func_id: FunctionId,
}

impl<'a> DataflowAnalyzer<'a> {
    pub fn new(db: &'a SemanticDB, func_id: FunctionId) -> Self {
        Self {
            db,
            states: HashMap::new(),
            errors: Vec::new(),
            has_returned: false,
            current_func_id: func_id,
        }
    }

    pub fn report_error(&mut self, span: Span, kind: Kind) {
        self.errors.push(HarpyError::new(span, kind));
    }

    fn is_copy_type(symbol: SymbolId, db: &SemanticDB) -> bool {
        let symbol = db.symbol_table.get(symbol);
        let ty = db.type_table.get(symbol.ty);
        match ty {
            ResolvedType::Boxed(_, _) => false,
            ResolvedType::Array(_, _) => false,
            ResolvedType::Str => false,
            ResolvedType::Struct(_) => false,
            ResolvedType::Ref(_, mutable) => !mutable,
            _ => true,
        }
    }

    pub fn is_mutable(&self, symbol: SymbolId) -> bool {
        let symbol = self.db.symbol_table.get(symbol);
        symbol.is_mutable
    }

    pub fn consume_value(&mut self, symbol: SymbolId, span: Span) {
        let fn_def = self.db.function_table.get(self.current_func_id);
        if !fn_def.locals.contains(&symbol) {
            return;
        }

        let state = self.states.entry(symbol).or_insert(VarState::Uninit);

        match state {
            VarState::Init => {
                if !Self::is_copy_type(symbol, self.db) {
                    *state = VarState::Moved;
                }
            }
            VarState::Moved => {
                self.report_error(span, Kind::UseOfMoved);
            }
            VarState::Uninit => {
                self.report_error(span, Kind::UseOfUninit);
            }
        }
    }

    pub fn assign_value(&mut self, symbol: SymbolId, span: Span) {
        let state = self.states.entry(symbol).or_insert(VarState::Uninit);
        match state {
            VarState::Uninit | VarState::Moved => {
                *state = VarState::Init;
            }
            VarState::Init => {
                if !self.is_mutable(symbol) {
                    self.report_error(span, Kind::CannotAssignImmutable);
                }
            }
        }
    }

    pub fn branch_state<F: FnOnce(&mut Self)>(
        &mut self,
        branch: F,
    ) -> (HashMap<SymbolId, VarState>, bool) {
        let original_states = self.states.clone();
        let original_return = self.has_returned;

        self.has_returned = false;
        branch(self);

        let branch_return = self.has_returned;
        let branch_states = std::mem::replace(&mut self.states, original_states);
        self.has_returned = original_return;

        (branch_states, branch_return)
    }
    pub fn merge_states(
        &mut self,
        true_states: HashMap<SymbolId, VarState>,
        false_states: HashMap<SymbolId, VarState>,
    ) {
        for (symbol, true_state) in true_states {
            let false_state = false_states.get(&symbol).unwrap_or(&VarState::Uninit);

            if true_state == *false_state {
                self.states.insert(symbol, true_state);
                continue;
            }

            if true_state == VarState::Uninit || *false_state == VarState::Uninit {
                self.states.insert(symbol, VarState::Uninit);
            } else if true_state == VarState::Moved || *false_state == VarState::Moved {
                self.states.insert(symbol, VarState::Moved);
            }
        }
    }

    pub fn get_base_symbol(&self, expr: &Node<Expr>) -> Option<SymbolId> {
        match &expr.inner {
            Expr::Ident(_) => self.db.symbol_table.resolutions.get(&expr.id).copied(),
            Expr::MemberAccess(expr, _) => self.get_base_symbol(expr),
            Expr::Index(expr, _) => self.get_base_symbol(expr),
            Expr::Borrow(expr, _) => self.get_base_symbol(expr),
            _ => None,
        }
    }

    pub fn check_valid_return(&mut self, expr: &Node<Expr>) {
        let Some(ret_ty_id) = self.db.type_table.is_cached(expr.id) else {
            return;
        };

        let ret_ty = self.db.type_table.get(ret_ty_id);

        if !matches!(ret_ty, ResolvedType::Ref(_, _)) {
            return;
        }

        if let Some(base) = self.get_base_symbol(expr) {
            let sym = self.db.symbol_table.get(base);
            if matches!(sym.kind, SymbolKind::Local | SymbolKind::Parameter) {
                self.report_error(expr.span, Kind::ReturnRefToLocal);
            }
        }
    }
}

impl Analyzer {
    pub fn dataflow_analysis(&mut self, ast: &Program) {
        for stmt in &ast.items {
            if let Stmt::Function(decl) = stmt {
                let errors = DataflowAnalyzer::check_function(&self.db, decl);
                self.errors.extend(errors);
            }
        }
    }
}
