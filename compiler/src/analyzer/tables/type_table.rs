use std::collections::HashMap;

use crate::{
    analyzer::types::types::{ResolvedType, TypeId},
    parser::node::NodeId,
};

#[macro_export]
macro_rules! get_ty {
    ($analyzer:tt($id:expr), $pattern:pat => $expected:expr) => {{
        use $crate::unwrap_variant;
        unwrap_variant!($analyzer.db.type_table.get($id), $pattern => $expected)
    }};
}

#[derive(Debug)]
pub struct TypeTable {
    lookup: HashMap<ResolvedType, TypeId>,
    node_types: HashMap<NodeId, TypeId>,
    types: Vec<ResolvedType>,
}

impl Default for TypeTable {
    fn default() -> Self {
        let mut table = Self {
            lookup: HashMap::new(),
            node_types: HashMap::new(),
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

    pub fn is_cached(&self, node: NodeId) -> Option<TypeId> {
        self.node_types.get(&node).copied()
    }

    pub fn cache(&mut self, node_id: NodeId, ty_id: TypeId) {
        self.node_types.insert(node_id, ty_id);
    }
}
