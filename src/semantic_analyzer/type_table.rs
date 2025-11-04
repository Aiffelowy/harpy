use std::collections::HashMap;

use crate::{aliases::TypeInfoRc, parser::types::Type};

use super::symbol_info::TypeInfo;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypeIndex(pub usize);

#[derive(Debug)]
pub struct TypeTable {
    pool: Vec<TypeInfoRc>,
    map: HashMap<Type, TypeIndex>,
}

impl TypeTable {
    pub(in crate::semantic_analyzer) fn new() -> Self {
        Self {
            pool: vec![],
            map: HashMap::new(),
        }
    }

    pub(in crate::semantic_analyzer) fn register(&mut self, ttype: &Type) -> TypeInfoRc {
        if let Some(info) = self.map.get(ttype) {
            return self.pool[info.0].clone();
        }

        let i = self.pool.len();
        let rc = TypeInfoRc::new(TypeInfo {
            ttype: ttype.clone(),
            size: ttype.calc_size(),
        });

        self.pool.push(rc.clone());
        self.map.insert(ttype.clone(), TypeIndex(i));

        rc
    }

    pub fn get(&self, id: TypeIndex) -> Option<TypeInfoRc> {
        self.pool.get(id.0).cloned()
    }
}
