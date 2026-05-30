use crate::{
    analyzer::{
        analyzer::SemanticDB,
        dataflow_pass::data_analyzer::{DataflowAnalyzer, VarState},
        types::types::{ResolvedType, TypeId},
    },
    err::{HarpyError, Kind},
    parser::{
        stmt::stmts::{FunctionDecl, Stmt},
        Node,
    },
};

impl<'a> DataflowAnalyzer<'a> {
    pub fn check_stmt(&mut self, stmt: &Node<Stmt>) {
        match &stmt.inner {
            Stmt::For(f) => {
                let (loop_states, _) = self.branch_state(|a| {
                    a.check_expr(&f.iter_expr);
                    a.check_block(&f.block);

                    a.check_expr(&f.iter_expr);
                    a.check_block(&f.block);
                });
                self.merge_states(self.states.clone(), loop_states);
            }
            Stmt::While(w) => {
                let (loop_states, _) = self.branch_state(|a| {
                    a.check_expr(&w.expr);
                    a.check_block(&w.block);

                    a.check_expr(&w.expr);
                    a.check_block(&w.block);
                });
                self.merge_states(self.states.clone(), loop_states);
            }

            Stmt::Let(l) => {
                let sym_id = self.db.symbol_table.resolutions[&l.id];

                if let Some(init_expr) = &l.expr {
                    self.check_expr(init_expr);

                    self.states.insert(sym_id, VarState::Init);
                } else {
                    self.states.insert(sym_id, VarState::Uninit);
                }
            }

            Stmt::Global(g) => {
                self.check_expr(&g.expr);
                let sym_id = self.db.symbol_table.resolutions[&stmt.id];
                self.states.insert(sym_id, VarState::Init);
            }

            Stmt::Semi(e) => {
                self.check_expr(e);
            }
            Stmt::Expr(e) => {
                self.check_expr(e);
            }
            Stmt::Struct(_) => {}
            Stmt::Function(f) => {
                let errors = Self::check_function(self.db, f);
                self.errors.extend(errors);
            }
        }
    }

    pub fn check_function(db: &SemanticDB, f: &Node<FunctionDecl>) -> Vec<HarpyError> {
        let fn_id = db.function_table.get_resolution(f.id);
        let fn_def = db.function_table.get(fn_id);

        let mut analyzer = DataflowAnalyzer::new(db, fn_id);
        for param_id in &fn_def.params {
            analyzer.states.insert(*param_id, VarState::Init);
        }

        analyzer.check_block(&f.block);
        let sig = fn_def.signature;
        let return_ty = match db.type_table.get(sig) {
            ResolvedType::Function { return_type, .. } => return_type,
            _ => unreachable!(),
        };

        if *return_ty != TypeId::void() && !analyzer.has_returned && f.block.tail.is_none() {
            analyzer.report_error(f.span, Kind::NotAllPathsReturn);
        }

        if let Some(e) = &f.block.tail {
            analyzer.check_valid_return(e);
        }

        analyzer.errors
    }
}
