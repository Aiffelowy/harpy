use std::collections::HashMap;

use crate::{
    aliases::{Result, SymbolInfoRef},
    err::HarpyError,
    extensions::SymbolInfoRefExt,
    lexer::tokens::Ident,
    parser::{
        expr::expr::CallExpr,
        node::{Node, NodeId},
        types::TypeInner,
    },
};

use super::{
    err::SemanticError,
    symbol_info::{RuntimeFunctionInfo, SymbolInfoKind},
    type_table::{RuntimeConversionTypeTable, RuntimeTypeIndex},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FuncIndex(pub u32);

#[derive(Debug)]
pub struct FunctionTable {
    pool: Vec<SymbolInfoRef>,
    map: HashMap<String, FuncIndex>,
    call_map: HashMap<NodeId, FuncIndex>,
    func_delc_map: HashMap<NodeId, FuncIndex>,
}

impl FunctionTable {
    pub fn new() -> Self {
        Self {
            pool: vec![],
            map: HashMap::new(),
            call_map: HashMap::new(),
            func_delc_map: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &Node<Ident>, info: SymbolInfoRef) -> FuncIndex {
        if let Some(idx) = self.map.get(name.value()) {
            return *idx;
        }

        //FIX!!!
        let idx = FuncIndex(self.pool.len().try_into().unwrap());
        self.pool.push(info.clone());
        self.map.insert(name.value().to_owned(), idx);
        self.func_delc_map.insert(name.id(), idx);
        idx
    }

    pub fn register_call(&mut self, name: &Ident, call_expr: &Node<CallExpr>) -> Option<()> {
        if let Some(idx) = self.map.get(name.value()) {
            self.call_map.insert(call_expr.id(), *idx);
            return Some(());
        }

        None
    }

    pub fn get(&self, idx: FuncIndex) -> SymbolInfoRef {
        self.pool[idx.0 as usize].clone()
    }

    pub(in crate::semantic_analyzer) fn into_runtime(
        self,
        type_table: &RuntimeConversionTypeTable,
    ) -> std::result::Result<RuntimeFunctionTable, Vec<HarpyError>> {
        let mut pool = Vec::with_capacity(self.pool.len());
        let mut error_pool = vec![];
        for info in self.pool {
            let info = info.get();
            if let SymbolInfoKind::Function(i) = &info.kind {
                let params = i
                    .params
                    .iter()
                    .map(|info| type_table.get_mapping(&info.idx))
                    .collect();
                let locals = i
                    .locals
                    .iter()
                    .map(|local| {
                        let local = &local.get();
                        let ty = &local.ty;
                        if ty.inner == TypeInner::Unknown {
                            return Err(HarpyError::new(
                                crate::err::HarpyErrorKind::SemanticError(
                                    SemanticError::CantInferType,
                                ),
                                local.span,
                            ));
                        }
                        Ok(type_table.get_mapping(&ty.idx))
                    })
                    .collect::<Vec<Result<RuntimeTypeIndex>>>();
                let return_type = type_table.get_mapping(&info.ty.idx);

                let (oks, errors): (Vec<_>, Vec<_>) = locals.into_iter().partition(Result::is_ok);
                let errs = errors
                    .into_iter()
                    .map(Result::unwrap_err)
                    .collect::<Vec<HarpyError>>();
                if !errs.is_empty() {
                    error_pool.extend(errs);
                    continue;
                }

                let locals = oks
                    .into_iter()
                    .map(Result::unwrap)
                    .collect::<Vec<RuntimeTypeIndex>>();

                pool.push(RuntimeFunctionInfo {
                    params,
                    locals,
                    return_type,
                });
            }
        }

        if !error_pool.is_empty() {
            return Err(error_pool);
        }

        Ok(RuntimeFunctionTable {
            pool,
            call_map: self.call_map,
            func_delc_map: self.func_delc_map,
        })
    }
}

#[derive(Debug)]
pub struct RuntimeFunctionTable {
    pool: Vec<RuntimeFunctionInfo>,
    call_map: HashMap<NodeId, FuncIndex>,
    func_delc_map: HashMap<NodeId, FuncIndex>,
}

impl RuntimeFunctionTable {
    pub fn get(&self, idx: FuncIndex) -> &RuntimeFunctionInfo {
        &self.pool[idx.0 as usize]
    }

    pub fn get_mapping(&self, idx: NodeId) -> FuncIndex {
        self.call_map[&idx]
    }

    pub fn get_function_delc_mapping(&self, id: NodeId) -> FuncIndex {
        self.func_delc_map[&id]
    }

    pub fn iter(&self) -> std::slice::Iter<'_, RuntimeFunctionInfo> {
        self.pool.iter()
    }
}
