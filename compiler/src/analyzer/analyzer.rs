use std::collections::HashMap;

use crate::{
    aliases::Result,
    analyzer::{
        err::SymbolDeclError,
        modules::{Module, ModuleId},
        symbols::symbols::{Symbol, SymbolId},
        tables::{
            const_pool::ConstPool,
            function_table::{FunctionDef, FunctionId, FunctionTable},
            global_table::GlobalTable,
            struct_table::{StructId, StructLayout, StructTable},
            type_table::TypeTable,
        },
        types::types::{ResolvedType, TypeId},
    },
    err::HarpyError,
    lexer::tokens::{Ident, Lit},
    parser::{
        expr::expr_defs::Expr,
        node::NodeId,
        types::type_parsing::{BaseType, TypeInner},
        Node,
    },
};

#[derive(Debug)]
pub struct SemanticDB {
    pub modules: Vec<Module>,

    pub symbols: Vec<Symbol>,
    pub resolutions: HashMap<NodeId, SymbolId>,
    pub node_types: HashMap<NodeId, TypeId>,

    pub type_table: TypeTable,
    pub struct_table: StructTable,

    pub const_pool: ConstPool,
    pub function_table: FunctionTable,
    pub global_table: GlobalTable,

    pub entry_point: Option<FunctionId>,
}

impl Default for SemanticDB {
    fn default() -> Self {
        Self {
            modules: vec![Module::new("global", None)],
            symbols: vec![],
            resolutions: HashMap::new(),
            node_types: HashMap::new(),
            type_table: TypeTable::default(),
            struct_table: StructTable::default(),
            const_pool: ConstPool::default(),
            function_table: FunctionTable::default(),
            global_table: GlobalTable::default(),

            entry_point: None,
        }
    }
}

#[derive(Debug, Default)]
pub struct Analyzer {
    db: SemanticDB,
}

impl Analyzer {
    fn resolve_type_base(
        &mut self,
        module_id: ModuleId,
        base_type: &Node<BaseType>,
    ) -> Result<TypeId> {
        let bt = match &base_type.inner {
            BaseType::Int => ResolvedType::Int,
            BaseType::Bool => ResolvedType::Bool,
            BaseType::Str => ResolvedType::Str,
            BaseType::Float => ResolvedType::Float,
            BaseType::Custom(name) => {
                if let Some(struct_id) = self.db.modules[module_id.0].structs.get(name.value()) {
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

    fn resolve_array_type(
        &mut self,
        module_id: ModuleId,
        ty: &TypeInner,
        expr: &Option<Box<Node<Expr>>>,
    ) -> Result<TypeId> {
        let resolved = self.resolve_type(module_id, ty)?;
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

    pub fn resolve_type(&mut self, module_id: ModuleId, parser_type: &TypeInner) -> Result<TypeId> {
        let ty = match parser_type {
            TypeInner::Base(base_node) => return self.resolve_type_base(module_id, base_node),
            TypeInner::Void => ResolvedType::Void,
            TypeInner::Boxed(inner) => {
                let resolved = self.resolve_type(module_id, &inner.inner)?;
                ResolvedType::Boxed(resolved)
            }

            TypeInner::FunctionType(f) => {
                let args = f
                    .args
                    .iter()
                    .map(|arg| self.resolve_type(module_id, &arg.inner.inner))
                    .collect::<Result<Vec<_>>>()?;
                let return_type = self.resolve_type(module_id, &f.return_type.inner.inner)?;
                ResolvedType::Function { args, return_type }
            }
            TypeInner::Array(ty, s) => return self.resolve_array_type(module_id, &ty.inner, s),
            TypeInner::Unknown => ResolvedType::Unknown,
        };

        Ok(self.db.type_table.register(ty))
    }

    pub fn register_struct(
        &mut self,
        module_id: ModuleId,
        layout: StructLayout,
    ) -> Result<StructId> {
        let name = layout.name.clone();
        let span = layout.span;

        if let Some(&existing_id) = self.db.modules[module_id.0].structs.get(&name) {
            let original_span = self.db.struct_table.get(existing_id).span;

            return HarpyError::analyzer(
                SymbolDeclError::AlreadyExists {
                    name,
                    original_def: original_span,
                }
                .into(),
                span,
            );
        }

        let struct_id = self.db.struct_table.register(layout);
        self.db.modules[module_id.0].structs.insert(name, struct_id);

        Ok(struct_id)
    }

    pub fn register_function(
        &mut self,
        module_id: ModuleId,
        def: FunctionDef,
    ) -> Result<FunctionId> {
        let name = def.name.clone();
        let span = def.span;

        if let Some(&existing_id) = self.db.modules[module_id.0].functions.get(&name) {
            let original_span = self.db.function_table.get(existing_id).span;

            return HarpyError::analyzer(
                SymbolDeclError::AlreadyExists {
                    name,
                    original_def: original_span,
                }
                .into(),
                span,
            );
        }

        let func_id = self.db.function_table.register(def);
        self.db.modules[module_id.0].functions.insert(name, func_id);

        Ok(func_id)
    }

    pub fn resolve_struct_name(&self, module_id: ModuleId, ident: &Ident) -> Result<StructId> {
        let module = &self.db.modules[module_id.0];

        if let Some(&id) = module.structs.get(ident.value()) {
            Ok(id)
        } else {
            HarpyError::analyzer(
                SymbolDeclError::UnknownType(ident.value().to_owned()).into(),
                ident.span(),
            )
        }
    }

    pub fn resolve_function_name(&self, module_id: ModuleId, ident: &Ident) -> Result<FunctionId> {
        let module = &self.db.modules[module_id.0];

        if let Some(&id) = module.functions.get(ident.value()) {
            Ok(id)
        } else {
            HarpyError::analyzer(
                SymbolDeclError::UnknownFunction(ident.value().to_owned()).into(),
                ident.span(),
            )
        }
    }
}
