use std::collections::HashMap;

use crate::{
    aliases::Result,
    analyzer::{
        err::SymbolDeclError,
        symbols::symbols::{Symbol, SymbolId},
        tables::{
            const_pool::ConstPool, function_table::FunctionTable, global_table::GlobalTable,
            struct_table::StructTable, type_table::TypeTable,
        },
        types::types::{ResolvedType, TypeId},
    },
    err::HarpyError,
    lexer::tokens::Lit,
    parser::{
        expr::expr_defs::Expr,
        node::NodeId,
        types::type_parsing::{BaseType, Primitive, Type, TypeInner},
        Node,
    },
};

pub struct SemanticDB {
    pub symbols: Vec<Symbol>,

    pub resolutions: HashMap<NodeId, SymbolId>,
    pub node_types: HashMap<NodeId, TypeId>,

    pub type_table: TypeTable,
    pub struct_table: StructTable,

    pub const_pool: ConstPool,
    pub function_table: FunctionTable,
    pub global_table: GlobalTable,
}

pub struct Analyzer {
    db: SemanticDB,
}

impl Analyzer {
    fn resolve_type_base(&mut self, base_type: &Node<BaseType>) -> Result<TypeId> {
        let bt = match &base_type.inner {
            BaseType::Base(Primitive::Int) => ResolvedType::Int,
            BaseType::Base(Primitive::Bool) => ResolvedType::Bool,
            BaseType::Base(Primitive::Str) => ResolvedType::Str,
            BaseType::Base(Primitive::Float) => ResolvedType::Float,
            BaseType::Custom(name) => {
                if let Some(struct_id) = self.db.struct_table.name_to_id.get(name.value()) {
                    ResolvedType::Struct(*struct_id)
                } else {
                    return HarpyError::analyzer(
                        SymbolDeclError::UnknownType(name.value().clone()).into(),
                        base_type.span,
                    );
                }
            }
        };

        Ok(self.db.type_table.register(bt))
    }

    fn resolve_array_type(&mut self, ty: &Type, expr: &Option<Box<Node<Expr>>>) -> Result<TypeId> {
        let resolved = self.resolve_type(&ty.inner)?;
        let mut size = None;
        if let Some(expr) = expr {
            match &expr.inner {
                Expr::Literal(lit) => {
                    if let Lit::LitInt(i) = lit.value() {
                        size = Some(*i);
                    } else {
                        return HarpyError::analyzer(
                            SymbolDeclError::ArraySizeInt.into(),
                            lit.span(),
                        );
                    }
                }
                _ => return HarpyError::analyzer(SymbolDeclError::ArraySizeInt.into(), expr.span),
            }
        }
        let ty = ResolvedType::Array(resolved, size);
        Ok(self.db.type_table.register(ty))
    }

    pub fn resolve_type(&mut self, parser_type: &TypeInner) -> Result<TypeId> {
        let ty = match parser_type {
            TypeInner::Base(base_node) => return self.resolve_type_base(base_node),
            TypeInner::Void => ResolvedType::Void,
            TypeInner::Ref(inner) => {
                let resolved = self.resolve_type(&inner.inner.inner)?;
                ResolvedType::Ref(resolved)
            }
            TypeInner::Boxed(inner) => {
                let resolved = self.resolve_type(&inner.inner.inner)?;
                ResolvedType::Boxed(resolved)
            }

            TypeInner::FunctionType(f) => {
                let args = f
                    .args
                    .iter()
                    .map(|arg| self.resolve_type(&arg.inner.inner))
                    .collect::<Result<Vec<_>>>()?;
                let return_type = self.resolve_type(&f.return_type.inner.inner)?;
                ResolvedType::Function { args, return_type }
            }
            TypeInner::Array(ty, s) => return self.resolve_array_type(&ty.inner, s),
            TypeInner::Unknown => ResolvedType::Unknown,
        };

        Ok(self.db.type_table.register(ty))
    }
}
