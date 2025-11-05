use std::collections::HashMap;

use crate::{
    aliases::{Result, TypeInfoRc},
    parser::types::{RuntimeType, Type},
};

use super::symbol_info::{RuntimeTypeInfo, TypeInfo};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypeIndex(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RuntimeTypeIndex(pub usize);

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
            idx: TypeIndex(i),
        });

        self.pool.push(rc.clone());
        self.map.insert(ttype.clone(), TypeIndex(i));

        rc
    }

    pub fn get(&self, id: TypeIndex) -> Option<TypeInfoRc> {
        self.pool.get(id.0).cloned()
    }

    pub fn into_runtime(self) -> Result<RuntimeTypeTable> {
        let mut runtime_type_table = RuntimeTypeTable::new();
        for ty in self.pool {
            runtime_type_table.register(ty)?;
        }

        Ok(runtime_type_table)
    }
}

#[derive(Debug)]
pub struct RuntimeTypeTable {
    pool: Vec<RuntimeTypeInfo>,
    map: HashMap<RuntimeType, RuntimeTypeIndex>,
    runtime_mapping: HashMap<TypeIndex, RuntimeTypeIndex>,
}

impl RuntimeTypeTable {
    fn new() -> Self {
        Self {
            pool: vec![],
            map: HashMap::new(),
            runtime_mapping: HashMap::new(),
        }
    }

    fn register(&mut self, type_info: TypeInfoRc) -> Result<()> {
        let runtime_info = type_info.into_runtime()?;
        let idx = RuntimeTypeIndex(self.pool.len());

        if self.map.contains_key(&runtime_info.ttype) {
            self.runtime_mapping.insert(type_info.idx, idx);
            return Ok(());
        }

        let rttc = runtime_info.ttype.clone();
        self.pool.push(runtime_info);
        self.map.insert(rttc, idx);
        self.runtime_mapping.insert(type_info.idx, idx);
        Ok(())
    }

    pub fn get_mapping(&self, type_idx: &TypeIndex) -> RuntimeTypeIndex {
        self.runtime_mapping[type_idx]
    }

    pub fn get(&self, type_idx: RuntimeTypeIndex) -> &RuntimeTypeInfo {
        &self.pool[type_idx.0]
    }
}
