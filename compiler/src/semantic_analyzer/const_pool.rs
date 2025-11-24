use std::collections::HashMap;

use crate::{
    aliases::TypeInfoRc,
    lexer::tokens::{Lit, Literal},
    parser::node::{Node, NodeId},
};

use super::type_table::{RuntimeConversionTypeTable, RuntimeTypeIndex, TypeIndex};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ConstIndex(pub u32);

#[derive(Debug, Clone)]
pub struct ConstInfo {
    lit: Lit,
    type_idx: TypeIndex,
}

#[derive(Debug, Clone)]
pub struct RuntimeConstInfo {
    pub lit: Lit,
    pub type_idx: RuntimeTypeIndex,
}

#[derive(Debug)]
pub struct ConstPool {
    pool: Vec<ConstInfo>,
    map: HashMap<Lit, ConstIndex>,
    node_map: HashMap<NodeId, ConstIndex>,
}

impl ConstPool {
    pub(in crate::semantic_analyzer) fn new() -> Self {
        let mut pool = vec![];
        let mut map = HashMap::new();

        pool.push(ConstInfo {
            lit: Lit::LitVoid,
            type_idx: TypeIndex(0),
        });
        map.insert(Lit::LitVoid, ConstIndex(0));

        Self {
            pool,
            map,
            node_map: HashMap::new(),
        }
    }

    pub fn register(&mut self, lit: &Node<Literal>, info: &TypeInfoRc) -> ConstIndex {
        if let Some(i) = self.map.get(lit.value()) {
            self.node_map.insert(lit.id(), *i);
            return *i;
        }

        //FIX!!!
        let i = ConstIndex(self.pool.len().try_into().unwrap());
        self.pool.push(ConstInfo {
            lit: lit.value().clone(),
            type_idx: info.idx,
        });
        self.map.insert(lit.value().clone(), i);
        self.node_map.insert(lit.id(), i);
        i
    }

    pub fn get(&self, id: ConstIndex) -> Option<&Lit> {
        self.pool.get(id.0 as usize).map(|c| &c.lit)
    }

    pub(in crate::semantic_analyzer) fn to_runtime(
        self,
        type_table: &RuntimeConversionTypeTable,
    ) -> RuntimeConstPool {
        let mut pool = Vec::with_capacity(self.pool.len());
        for entry in self.pool {
            pool.push(RuntimeConstInfo {
                lit: entry.lit,
                type_idx: type_table.get_mapping(&entry.type_idx),
            });
        }

        RuntimeConstPool {
            pool,
            node_map: self.node_map,
        }
    }
}

#[derive(Debug)]
pub struct RuntimeConstPool {
    pool: Vec<RuntimeConstInfo>,
    node_map: HashMap<NodeId, ConstIndex>,
}

impl RuntimeConstPool {
    pub fn get(&self, idx: ConstIndex) -> &RuntimeConstInfo {
        &self.pool[idx.0 as usize]
    }

    pub fn get_mapping(&self, idx: NodeId) -> ConstIndex {
        self.node_map[&idx]
    }

    pub fn iter(&self) -> std::slice::Iter<'_, RuntimeConstInfo> {
        self.pool.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        aliases::TypeInfoRc,
        lexer::tokens::Lit,
        lexer::span::Span,
        parser::{node::Node, types::{Type}},
        semantic_analyzer::type_table::{TypeTable, TypeIndex},
    };
    use std::{rc::Rc};

    fn create_type_info(ty: Type, idx: TypeIndex) -> TypeInfoRc {
        Rc::new(crate::semantic_analyzer::symbol_info::TypeInfo {
            size: ty.calc_size(),
            ttype: ty,
            idx,
        })
    }

    fn literal(lit: Lit) -> Node<Literal> {
        Node::dummy(Literal { span: Span::default(), value: lit })
    }

    #[test]
    fn test_const_pool_initialization() {
        let pool = ConstPool::new();
        assert_eq!(pool.pool.len(), 1);
        assert!(pool.map.contains_key(&Lit::LitVoid));
        assert_eq!(pool.get(ConstIndex(0)), Some(&Lit::LitVoid));
    }

    #[test]
    fn test_const_pool_register_literals() {
        let mut pool = ConstPool::new();
        
        let int_literal = literal(Lit::LitInt(42));
        let float_literal = literal(Lit::LitFloat(3.14f64.to_bits()));
        let bool_literal = literal(Lit::LitBool(true));
        let str_literal = literal(Lit::LitStr("hello".to_string()));
        
        let int_type = create_type_info(Type::int(), TypeIndex(1));
        let float_type = create_type_info(Type::float(), TypeIndex(2));
        let bool_type = create_type_info(Type::bool(), TypeIndex(3));
        let str_type = create_type_info(Type::str(), TypeIndex(4));
        
        let idx1 = pool.register(&int_literal, &int_type);
        let idx2 = pool.register(&float_literal, &float_type);
        let idx3 = pool.register(&bool_literal, &bool_type);
        let idx4 = pool.register(&str_literal, &str_type);
        
        assert_eq!(idx1, ConstIndex(1));
        assert_eq!(idx2, ConstIndex(2));
        assert_eq!(idx3, ConstIndex(3));
        assert_eq!(idx4, ConstIndex(4));
        
        assert_eq!(pool.get(idx1), Some(&Lit::LitInt(42)));
        assert_eq!(pool.get(idx2), Some(&Lit::LitFloat(3.14f64.to_bits())));
        assert_eq!(pool.get(idx3), Some(&Lit::LitBool(true)));
        assert_eq!(pool.get(idx4), Some(&Lit::LitStr("hello".to_string())));
    }

    #[test]
    fn test_const_pool_duplicate_registration() {
        let mut pool = ConstPool::new();
        
        let literal1 = literal(Lit::LitInt(42));
        let literal2 = literal(Lit::LitInt(42));
        
        let int_type = create_type_info(Type::int(), TypeIndex(1));
        
        let idx1 = pool.register(&literal1, &int_type);
        let idx2 = pool.register(&literal2, &int_type);
        
        assert_eq!(idx1, idx2);
        assert_eq!(pool.pool.len(), 2);
    }

    #[test]
    fn test_const_pool_to_runtime_conversion() {
        let mut pool = ConstPool::new();
        let mut type_table = TypeTable::new();
        
        let int_literal = literal(Lit::LitInt(42));
        let int_type_info = type_table.register(&Type::int());
        
        let const_idx = pool.register(&int_literal, &int_type_info);
        
        let runtime_type_table = type_table.into_conversion().unwrap();
        let runtime_pool = pool.to_runtime(&runtime_type_table);
        
        let runtime_const = runtime_pool.get(const_idx);
        assert_eq!(runtime_const.lit, Lit::LitInt(42));
        
        let mapped_idx = runtime_pool.get_mapping(int_literal.id());
        assert_eq!(mapped_idx, const_idx);
    }

    #[test]
    fn test_const_pool_integration_with_multiple_types() {
        let mut pool = ConstPool::new();
        let mut type_table = TypeTable::new();
        
        let literals = vec![
            (literal(Lit::LitInt(100)), Type::int()),
            (literal(Lit::LitFloat(2.71f64.to_bits())), Type::float()),
            (literal(Lit::LitBool(false)), Type::bool()),
            (literal(Lit::LitStr("world".to_string())), Type::str()),
        ];
        
        let mut indices = Vec::new();
        for (literal, ty) in &literals {
            let type_info = type_table.register(ty);
            let const_idx = pool.register(literal, &type_info);
            indices.push(const_idx);
        }
        
        let runtime_type_table = type_table.into_conversion().unwrap();
        let runtime_pool = pool.to_runtime(&runtime_type_table);
        
        for (i, (literal, _)) in literals.iter().enumerate() {
            let runtime_const = runtime_pool.get(indices[i]);
            assert_eq!(runtime_const.lit, *literal.value());
        }
    }
}
