use std::collections::HashMap;

use crate::{
    aliases::{Result, TypeInfoRc},
    parser::types::{RuntimeType, Type, TypeInner},
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

        match &ttype.inner {
            TypeInner::Boxed(b) | TypeInner::Ref(b) => {
                self.register(b);
            }
            _ => (),
        }

        let i = self.pool.len();
        let rc = TypeInfoRc::new(
            TypeInfo {
                ttype: ttype.clone(),
                size: ttype.calc_size(),
                idx: TypeIndex(i),
            }
            .into(),
        );

        self.pool.push(rc.clone());
        self.map.insert(ttype.clone(), TypeIndex(i));

        rc
    }

    pub(in crate::semantic_analyzer) fn into_conversion(
        self,
    ) -> Result<RuntimeConversionTypeTable> {
        let mut runtime_type_table = RuntimeConversionTypeTable::new(self.map);
        for ty in self.pool {
            runtime_type_table.register(ty)?;
        }

        Ok(runtime_type_table)
    }
}

#[derive(Debug)]
pub struct RuntimeConversionTypeTable {
    pool: Vec<RuntimeTypeInfo>,
    map: HashMap<RuntimeType, RuntimeTypeIndex>,
    old_map: HashMap<Type, TypeIndex>,
    runtime_mapping: HashMap<TypeIndex, RuntimeTypeIndex>,
}

impl RuntimeConversionTypeTable {
    fn new(old_map: HashMap<Type, TypeIndex>) -> Self {
        Self {
            pool: vec![],
            map: HashMap::new(),
            runtime_mapping: HashMap::new(),
            old_map,
        }
    }

    fn register(&mut self, type_info: TypeInfoRc) -> Result<()> {
        let runtime_info = type_info.into_runtime(self)?;

        if let Some(idx) = self.map.get(&runtime_info.ttype) {
            self.runtime_mapping.insert(type_info.idx, *idx);
            return Ok(());
        }

        let idx = RuntimeTypeIndex(self.pool.len());
        let rttc = runtime_info.ttype.clone();
        self.pool.push(runtime_info);
        self.map.insert(rttc, idx);
        self.runtime_mapping.insert(type_info.idx, idx);
        Ok(())
    }

    pub(in crate::semantic_analyzer) fn get_mapping(
        &self,
        type_idx: &TypeIndex,
    ) -> RuntimeTypeIndex {
        self.runtime_mapping[type_idx]
    }

    pub(in crate::semantic_analyzer) fn get_type_index(&self, ty: &Type) -> TypeIndex {
        self.old_map[ty]
    }

    pub(in crate::semantic_analyzer) fn into_runtime(self) -> RuntimeTypeTable {
        RuntimeTypeTable { pool: self.pool }
    }
}

#[derive(Debug)]
pub struct RuntimeTypeTable {
    pool: Vec<RuntimeTypeInfo>,
}

impl RuntimeTypeTable {
    pub fn get(&self, type_idx: RuntimeTypeIndex) -> &RuntimeTypeInfo {
        &self.pool[type_idx.0]
    }
}
