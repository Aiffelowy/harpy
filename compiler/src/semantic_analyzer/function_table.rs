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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        aliases::{SymbolInfoRef, TypeInfoRc},
        extensions::{SymbolInfoRefExt},
        lexer::{span::Span, tokens::Ident},
        parser::{expr::expr::CallExpr, node::Node, types::{Type}},
        semantic_analyzer::{
            scope::Depth, symbol_info::{FunctionInfo, SymbolInfo, SymbolInfoKind, VariableInfo}, type_table::{TypeTable }
        },
    };
    use std::{cell::RefCell, rc::Rc};

    fn create_function_symbol(node_id: NodeId, params: Vec<TypeInfoRc>, locals: Vec<SymbolInfoRef>, return_type: TypeInfoRc) -> SymbolInfoRef {
        let symbol_info = SymbolInfo {
            kind: SymbolInfoKind::Function(FunctionInfo { params, locals }),
            ty: return_type,
            node_id,
            ref_count: 0,
            scope_depth: Depth(0),
            span: Span::default(),
        };
        Rc::new(RefCell::new(symbol_info))
    }

    fn create_local_variable(node_id: NodeId, type_info: TypeInfoRc) -> SymbolInfoRef {
        let symbol_info = SymbolInfo {
            kind: SymbolInfoKind::Variable(VariableInfo {
                initialized: true,
                mutably_borrowed: false,
                immutably_borrowed_count: 0,
            }),
            ty: type_info,
            node_id,
            ref_count: 0,
            scope_depth: Depth(0),
            span: Span::default(),
        };
        Rc::new(RefCell::new(symbol_info))
    }

    fn dummy_ident(name: &str) -> Node<Ident> {
        Node::dummy(Ident { span: Span::default(), value: name.to_string() })
    }

    #[test]
    fn test_function_table_initialization() {
        let table = FunctionTable::new();
        assert_eq!(table.pool.len(), 0);
        assert!(table.map.is_empty());
        assert!(table.call_map.is_empty());
        assert!(table.func_delc_map.is_empty());
    }

    #[test]
    fn test_function_table_register_function() {
        let mut table = FunctionTable::new();
        let mut type_table = TypeTable::new();
        
        let func_name = dummy_ident("test_func");
        let int_type = Type::int();
        let int_type = type_table.register(&int_type);
        
        let params = vec![int_type.clone()];
        let locals = vec![];
        
        let function = create_function_symbol(NodeId(0), params, locals, int_type);
        let func_idx = table.register(&func_name, function.clone());
        
        assert_eq!(func_idx, FuncIndex(0));
        assert_eq!(table.pool.len(), 1);
        
        let retrieved = table.get(func_idx);
        let func_info = retrieved.as_function().unwrap();
        assert_eq!(func_info.params.len(), 1);
        assert_eq!(func_info.locals.len(), 0);
    }

    #[test]
    fn test_function_table_register_call() {
        let mut table = FunctionTable::new();
        let mut type_table = TypeTable::new();
        
        let func_name = dummy_ident("my_func");
        let call_expr = Node::dummy(CallExpr {
            ident: (*func_name).clone(),
            args: vec![],
        });
        
        let int_type = type_table.register(&Type::int());
        
        let function = create_function_symbol(NodeId(0), vec![], vec![], int_type);
        let _func_idx = table.register(&func_name, function);
        
        let result = table.register_call(&func_name, &call_expr);
        assert!(result.is_some());
        
        let non_existent_call = Node::dummy(CallExpr {
            ident: (*dummy_ident("non_existent")).clone(),
            args: vec![],
        });
        let result2 = table.register_call(&Ident { span: Span::default(), value: "non_existent".to_string() }, &non_existent_call);
        assert!(result2.is_none());
    }

    #[test]
    fn test_function_table_with_params_and_locals() {
        let mut table = FunctionTable::new();
        let mut type_table = TypeTable::new();
        
        let func_name = dummy_ident("complex_func");
        
        let int_type = type_table.register(&Type::int());
        let float_type = type_table.register(&Type::float());
        
        let params = vec![int_type.clone(), float_type.clone()];
        
        let locals = vec![
            create_local_variable(NodeId(1), int_type.clone()),
            create_local_variable(NodeId(2), float_type.clone()),
        ];
        
        let function = create_function_symbol(NodeId(0), params, locals, int_type);
        let func_idx = table.register(&func_name, function);
        
        let retrieved = table.get(func_idx);
        let func_info = retrieved.as_function().unwrap();
        
        assert_eq!(func_info.params.len(), 2);
        assert_eq!(func_info.locals.len(), 2);
    }

    #[test]
    fn test_function_table_to_runtime_conversion() {
        let mut table = FunctionTable::new();
        let mut type_table = TypeTable::new();
        
        let func_name = dummy_ident("runtime_func");
        
        let int_type = type_table.register(&Type::int());
        
        let params = vec![int_type.clone()];
        let locals = vec![create_local_variable(NodeId(1), int_type.clone())];
        
        let function = create_function_symbol(NodeId(0), params, locals, int_type);
        let func_idx = table.register(&func_name, function);
        
        let runtime_type_table = type_table.into_conversion().unwrap();
        let runtime_table = table.into_runtime(&runtime_type_table).unwrap();
        
        assert_eq!(runtime_table.pool.len(), 1);
        
        let runtime_func = runtime_table.get(func_idx);
        assert_eq!(runtime_func.params.len(), 1);
        assert_eq!(runtime_func.locals.len(), 1);
        assert_eq!(runtime_func.return_type.0, 1);
    }

    #[test]
    fn test_function_table_runtime_conversion_with_unknown_type() {
        let mut table = FunctionTable::new();
        let mut type_table = TypeTable::new();
        
        let func_name = dummy_ident("unknown_func");
        
        let unknown_type = type_table.register(&Type::unknown());
        let void_type = type_table.register(&Type::void());
        
        let locals = vec![create_local_variable(NodeId(1), unknown_type)];
        
        let function = create_function_symbol(NodeId(0), vec![], locals, void_type);
        table.register(&func_name, function);
        
        let runtime_type_table = type_table.into_conversion().unwrap();
        let result = table.into_runtime(&runtime_type_table);
        
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_function_table_multiple_functions() {
        let mut table = FunctionTable::new();
        let mut type_table = TypeTable::new();
        
        let functions = vec![
            ("add", Type::int()),
            ("multiply", Type::int()),
            ("print", Type::void()),
        ];
        
        let mut indices = Vec::new();
        for (i, (name, return_type)) in functions.iter().enumerate() {
            let func_name = dummy_ident(name);
            let return_type_info = type_table.register(return_type);
            
            let function = create_function_symbol(NodeId(i), vec![], vec![], return_type_info);
            let func_idx = table.register(&func_name, function);
            indices.push(func_idx);
        }
        
        assert_eq!(table.pool.len(), 3);
        
        for (i, func_idx) in indices.iter().enumerate() {
            assert_eq!(*func_idx, FuncIndex(i as u32));
            let retrieved = table.get(*func_idx);
            let func_info = retrieved.as_function().unwrap();
            assert_eq!(func_info.params.len(), 0);
        }
    }
}
