use crate::analyzer::{
    analyzer::SemanticDB,
    types::types::{ResolvedType, TypeId},
};

impl TypeId {
    pub(super) fn format(&self, db: &SemanticDB) -> String {
        db.type_table.get(*self).to_string(db)
    }
}

impl ResolvedType {
    fn to_string(&self, db: &SemanticDB) -> String {
        match self {
            ResolvedType::Int => "int".to_string(),
            ResolvedType::Float => "float".to_string(),
            ResolvedType::Bool => "bool".to_string(),
            ResolvedType::Str => "str".to_string(),
            ResolvedType::Void => "void".to_string(),
            ResolvedType::Never => "never".to_string(),
            ResolvedType::Unknown => "unknown".to_string(),

            ResolvedType::Ref(id, m) => {
                let m_str = if *m { "mut " } else { "" };
                let inner = id.format(db);
                format!("ref {}{}", m_str, inner)
            }
            ResolvedType::Boxed(id, m) => {
                let m_str = if *m { "mut " } else { "" };
                let inner = id.format(db);
                format!("boxed {}{}", m_str, inner)
            }

            ResolvedType::Array(id, size) => {
                let inner = id.format(db);
                if let Some(s) = size {
                    format!("[{}:{}]", inner, s)
                } else {
                    format!("[{}]", inner)
                }
            }

            ResolvedType::Range(inner_id) => {
                let inner = inner_id.format(db);
                format!("range {}", inner)
            }

            ResolvedType::Struct(struct_id) => {
                let struct_def = db.struct_table.get(*struct_id);
                struct_def.name.clone()
            }

            ResolvedType::Function { args, return_type } => {
                let args_str = args
                    .iter()
                    .map(|arg| arg.format(db))
                    .collect::<Vec<_>>()
                    .join(", ");

                let ret_str = return_type.format(db);

                format!("fn({}) -> {}", args_str, ret_str)
            }
        }
    }
}
