use std::collections::HashMap;

use crate::{
    aliases::SymbolInfoRef,
    extensions::SymbolInfoRefExt,
    lexer::tokens::Ident,
    parser::{
        expr::expr::CallExpr,
        node::{Node, NodeId},
    },
};

use super::{
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
    ) -> RuntimeFunctionTable {
        let mut pool = Vec::with_capacity(self.pool.len());
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
                        let ty = &local.get().ty;
                        type_table.get_mapping(&ty.idx)
                    })
                    .collect::<Vec<RuntimeTypeIndex>>();
                let return_type = type_table.get_mapping(&info.ty.idx);

                pool.push(RuntimeFunctionInfo {
                    params,
                    locals,
                    return_type,
                });
            }
        }

        RuntimeFunctionTable {
            pool,
            call_map: self.call_map,
            func_delc_map: self.func_delc_map,
        }
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
}
