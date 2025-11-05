use std::collections::HashMap;

use crate::{aliases::TypeInfoRc, lexer::tokens::Lit};

use super::type_table::{RuntimeConversionTypeTable, RuntimeTypeIndex, TypeIndex};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ConstIndex(pub usize);

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
}

impl ConstPool {
    pub(in crate::semantic_analyzer) fn new() -> Self {
        Self {
            pool: vec![],
            map: HashMap::new(),
        }
    }

    pub fn register(&mut self, lit: Lit, info: &TypeInfoRc) -> ConstIndex {
        if let Some(i) = self.map.get(&lit) {
            return *i;
        }

        let i = ConstIndex(self.pool.len());
        self.pool.push(ConstInfo {
            lit: lit.clone(),
            type_idx: info.idx,
        });
        self.map.insert(lit, i);
        i
    }

    pub fn get(&self, id: ConstIndex) -> Option<&Lit> {
        self.pool.get(id.0).map(|c| &c.lit)
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

        RuntimeConstPool { pool }
    }
}

#[derive(Debug)]
pub struct RuntimeConstPool {
    pool: Vec<RuntimeConstInfo>,
}

impl RuntimeConstPool {
    pub fn get(&self, idx: ConstIndex) -> &RuntimeConstInfo {
        &self.pool[idx.0]
    }
}
