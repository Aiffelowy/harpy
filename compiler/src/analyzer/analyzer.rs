use crate::{
    aliases::Result,
    analyzer::{
        err::{AnalyzerError, SymbolDeclError},
        modules::{Module, ModuleId},
        symbol_res::Environment,
        tables::{
            const_pool::ConstPool,
            function_table::{FunctionDef, FunctionId, FunctionTable},
            global_table::{GlobalDef, GlobalId, GlobalTable},
            struct_table::{StructId, StructLayout, StructTable},
            symbol_table::{Symbol, SymbolId, SymbolTable},
            type_table::TypeTable,
        },
        types::types::ResolvedType,
    },
    err::HarpyError,
    lexer::tokens::Ident,
    parser::stmt::stmts::Program,
};

#[macro_export]
macro_rules! unwrap_variant {
    ($expr:expr, $pattern:pat => $extracted:expr) => {
        match $expr {
            $pattern => $extracted,
            _ => unreachable!(
                "Expected pattern `{}` but got a different variant!",
                stringify!($pattern)
            ),
        }
    };
}

#[macro_export]
macro_rules! report {
    ($analyzer:expr, $expr:expr) => {
        match $expr {
            Ok(val) => Some(val),
            Err(e) => {
                $analyzer.errors.push(*e);
                None
            }
        }
    };
}

#[macro_export]
macro_rules! attempt {
    ($analyzer:tt.$($expr:tt)+) => {
        match $analyzer.$($expr)+ {
            Ok(val) => val,
            Err(e) => {
                use $crate::analyzer::analyzer::Fallback;
                $analyzer.errors.push(*e);
                $analyzer.fallback()
            }
        }
    };
}

pub trait Fallback<T> {
    fn fallback(&self) -> T;
}

#[derive(Debug)]
pub struct SemanticDB {
    pub modules: Vec<Module>,

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

#[derive(Debug)]
struct AnalyzerContext {
    current_module: ModuleId,
}

impl Default for AnalyzerContext {
    fn default() -> Self {
        Self {
            current_module: ModuleId(0),
        }
    }
}

#[derive(Debug, Default)]
pub struct Analyzer {
    pub(in crate::analyzer) db: SemanticDB,
    pub(in crate::analyzer) errors: Vec<HarpyError>,
    ctx: AnalyzerContext,
}

impl Analyzer {
    pub(in crate::analyzer) fn current_module(&self) -> &Module {
        &self.db.modules[self.ctx.current_module.0]
    }
    pub(in crate::analyzer) fn current_module_mut(&mut self) -> &mut Module {
        &mut self.db.modules[self.ctx.current_module.0]
    }

    pub(in crate::analyzer) fn register_struct(
        &mut self,
        layout: StructLayout,
        env: Option<&mut Environment>,
    ) -> Result<StructId> {
        let name = layout.name.clone();
        let span = layout.span;

        if let Some(ref env) = env {
            if env.has_type_in_current_scope(&name) {
                return HarpyError::analyzer(
                    SymbolDeclError::AlreadyExists {
                        name,
                        original_def: span,
                    }
                    .into(),
                    span,
                );
            }
        } else {
            if let Some(&existing_id) = self.current_module().structs.get(&name) {
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
        }

        let struct_id = self.db.struct_table.register(layout);

        let resolved_ty = ResolvedType::Struct(struct_id);
        let type_id = self.db.type_table.register(resolved_ty);

        if let Some(env) = env {
            env.declare_type(name, type_id);
        } else {
            self.current_module_mut().structs.insert(name, struct_id);
        }

        Ok(struct_id)
    }

    pub(in crate::analyzer) fn register_function(
        &mut self,
        def: FunctionDef,
    ) -> Result<FunctionId> {
        let name = def.name.clone();
        let span = def.span;

        if let Some(&existing_id) = self.current_module().functions.get(&name) {
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
        self.current_module_mut().functions.insert(name, func_id);

        Ok(func_id)
    }

    pub(in crate::analyzer) fn register_symbol(&mut self, symbol: Symbol) -> Result<SymbolId> {
        let id = self.db.symbol_table.register(symbol);
        Ok(id)
    }

    pub(in crate::analyzer) fn register_global(&mut self, def: GlobalDef) -> Result<GlobalId> {
        let name = def.name.clone();
        let span = def.span;

        if let Some(&existing_id) = self.current_module().globals.get(&name) {
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
        self.current_module_mut().globals.insert(name, global_id);
        Ok(global_id)
    }

    pub(in crate::analyzer) fn resolve_global_name(&self, ident: &Ident) -> Result<GlobalId> {
        let module = self.current_module();

        if let Some(&id) = module.globals.get(ident.value()) {
            Ok(id)
        } else {
            HarpyError::analyzer(
                SymbolDeclError::UnknownSymbol(ident.value().to_owned()).into(),
                ident.span(),
            )
        }
    }

    pub(in crate::analyzer) fn resolve_struct_name(&self, ident: &Ident) -> Result<StructId> {
        let module = self.current_module();

        if let Some(&id) = module.structs.get(ident.value()) {
            Ok(id)
        } else {
            HarpyError::analyzer(
                SymbolDeclError::UnknownType(ident.value().to_owned()).into(),
                ident.span(),
            )
        }
    }

    pub(in crate::analyzer) fn resolve_function_name(&self, ident: &Ident) -> Result<FunctionId> {
        let module = self.current_module();

        if let Some(&id) = module.functions.get(ident.value()) {
            Ok(id)
        } else {
            HarpyError::analyzer(
                SymbolDeclError::UnknownFunction(ident.value().to_owned()).into(),
                ident.span(),
            )
        }
    }

    pub(in crate::analyzer) fn report_error(
        &mut self,
        err: AnalyzerError,
        span: crate::lexer::span::Span,
    ) {
        self.errors.push(HarpyError::new_analyzer(err, span))
    }

    pub fn analyze(mut self, ast: &Program) -> std::result::Result<SemanticDB, Vec<HarpyError>> {
        self.pass_symbol_declaration(ast);
        self.symbol_resolution_pass(ast);

        if !self.errors.is_empty() {
            return Err(self.errors);
        }

        Ok(self.db)
    }
}
