use std::collections::HashMap;

use crate::analyzer::types::types::{ResolvedType, TypeId};

#[derive(Debug)]
pub struct TypeTable {
    lookup: HashMap<ResolvedType, TypeId>,
    types: Vec<ResolvedType>,
}

impl Default for TypeTable {
    fn default() -> Self {
        let mut table = Self {
            lookup: HashMap::new(),
            types: Vec::new(),
        };

        table.register(ResolvedType::Unknown);
        table.register(ResolvedType::Int);
        table.register(ResolvedType::Float);
        table.register(ResolvedType::Bool);
        table.register(ResolvedType::Str);
        table.register(ResolvedType::Void);
        table.register(ResolvedType::Never);
        table
    }
}

impl TypeTable {
    pub fn register(&mut self, ty: ResolvedType) -> TypeId {
        if let Some(&id) = self.lookup.get(&ty) {
            return id;
        }

        let new_id = TypeId(self.types.len());
        self.lookup.insert(ty.clone(), new_id);
        self.types.push(ty);
        new_id
    }

    pub fn get(&self, id: TypeId) -> &ResolvedType {
        &self.types[id.0]
    }
}
