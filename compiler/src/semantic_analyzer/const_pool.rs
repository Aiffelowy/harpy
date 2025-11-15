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
