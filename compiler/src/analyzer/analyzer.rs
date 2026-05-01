use std::collections::HashMap;

use crate::{
    aliases::Result,
    analyzer::{
        err::SymbolDeclError,
        modules::{Module, ModuleId},
        tables::{
            const_pool::ConstPool,
            function_table::{FunctionDef, FunctionId, FunctionTable},
            global_table::{GlobalDef, GlobalId, GlobalTable},
            struct_table::{StructId, StructLayout, StructTable},
            symbol_table::{Symbol, SymbolId, SymbolTable},
            type_table::TypeTable,
        },
        types::types::TypeId,
    },
    err::HarpyError,
    lexer::tokens::Ident,
    parser::{node::NodeId, stmt::stmts::Program},
};

#[derive(Debug)]
pub struct SemanticDB {
    pub modules: Vec<Module>,

    pub node_types: HashMap<NodeId, TypeId>,

    pub symbol_table: SymbolTable,
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
            node_types: HashMap::new(),
            symbol_table: SymbolTable::default(),
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
    pub(in crate::analyzer) db: SemanticDB,
}

impl Analyzer {
    pub(in crate::analyzer) fn register_struct(
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

    pub(in crate::analyzer) fn register_function(
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

    pub(in crate::analyzer) fn register_symbol(&mut self, symbol: Symbol) -> Result<SymbolId> {
        let id = self.db.symbol_table.register(symbol);
        Ok(id)
    }

    pub(in crate::analyzer) fn register_global(
        &mut self,
        module_id: ModuleId,
        def: GlobalDef,
    ) -> Result<GlobalId> {
        let name = def.name.clone();
        let span = def.span;

        if let Some(&existing_id) = self.db.modules[module_id.0].globals.get(&name) {
            let orig_span = self.db.global_table.get(existing_id).span;

            return HarpyError::analyzer(
                SymbolDeclError::AlreadyExists {
                    name,
                    original_def: orig_span,
                }
                .into(),
                span,
            );
        }

        let global_id = self.db.global_table.register(def);
        self.db.modules[module_id.0].globals.insert(name, global_id);
        Ok(global_id)
    }

    pub(in crate::analyzer) fn resolve_global_name(
        &self,
        module_id: ModuleId,
        ident: &Ident,
    ) -> Result<GlobalId> {
        let module = &self.db.modules[module_id.0];

        if let Some(&id) = module.globals.get(ident.value()) {
            Ok(id)
        } else {
            HarpyError::analyzer(
                SymbolDeclError::UnknownSymbol(ident.value().to_owned()).into(),
                ident.span(),
            )
        }
    }

    pub(in crate::analyzer) fn resolve_struct_name(
        &self,
        module_id: ModuleId,
        ident: &Ident,
    ) -> Result<StructId> {
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

    pub(in crate::analyzer) fn resolve_function_name(
        &self,
        module_id: ModuleId,
        ident: &Ident,
    ) -> Result<FunctionId> {
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

    pub fn analyze(mut self, ast: &Program) -> Result<SemanticDB> {
        self.pass_symbol_declaration(ast, ModuleId(0))?;

        Ok(self.db)
    }
}
