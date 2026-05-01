use crate::{
    analyzer::{
        analyzer::Analyzer,
        err::SymbolDeclError,
        symbol_res::Environment,
        tables::{
            function_table::FunctionDef,
            global_table::GlobalDef,
            struct_table::{Field, StructLayout},
            symbol_table::Symbol,
        },
        types::types::ResolvedType,
    },
    attempt, get_ty,
    parser::{
        stmt::stmts::{FunctionDecl, GlobalStmt, Program, Stmt, StructDecl},
        types::type_parsing::FunctionType,
        Node,
    },
};

impl Analyzer {
    fn pass_skeleton(&mut self, ast: &Program) {
        for stmt in &ast.items {
            if let Stmt::Struct(decl) = &stmt {
                let layout = StructLayout::skeleton(decl.name.value().clone(), decl.name.span());
                attempt!(self.register_struct(layout, None));
            }
        }
    }

    pub(in crate::analyzer) fn register_struct_fields(
        &mut self,
        decl: &StructDecl,
        env: Option<&Environment>,
    ) {
        let mut fields = vec![];
        for field in &decl.fields {
            let ty_id = attempt!(self.resolve_type_with_env(&field.ttype, env));
            fields.push(Field {
                name: field.name.value().clone(),
                ty: ty_id,
            });
        }

        let struct_id = if let Some(e) = env.and_then(|e| e.resolve_local_type(decl.name.value())) {
            get_ty!(self(e), ResolvedType::Struct(id) => *id)
        } else {
            attempt!(self.resolve_struct_name(&decl.name))
        };
        if !struct_id.is_valid() {
            return;
        }
        self.db.struct_table.get_mut(struct_id).set_fields(fields);
    }

    pub(in crate::analyzer) fn register_function_decl(
        &mut self,
        decl: &FunctionDecl,
        mut env: Option<&mut Environment>,
    ) {
        let args = decl.args.iter().map(|arg| arg.ty.clone()).collect();
        let fn_ty = FunctionType {
            args,
            return_type: Box::new(decl.return_type.clone()),
        };
        let signature = attempt!(self.resolve_fn_type(&fn_ty, env.as_deref()));
        if !signature.is_valid() {
            return;
        }
        let def = FunctionDef::skeleton(decl.name.value().clone(), signature, decl.name.span());
        let func_id = attempt!(self.register_function(def));

        let symbol = Symbol {
            id: None,
            name: decl.name.value().clone(),
            ty: signature,
            is_mutable: false,
            declared_at: decl.name.span(),
        };
        let symbol_id = attempt!(self.register_symbol(symbol));

        if symbol_id.is_valid() {
            if let Some(ref mut env) = env {
                env.declare_local(decl.name.value().clone(), symbol_id);
            } else {
                let global_def = GlobalDef {
                    id: None,
                    name: decl.name.value().clone(),
                    symbol: symbol_id,
                    span: decl.name.span(),
                };
                attempt!(self.register_global(global_def));
            }
        }

        if env.is_none() && decl.name.value() == "main" {
            self.db.entry_point = Some(func_id)
        }
    }

    fn register_global_decl(&mut self, decl: &Node<GlobalStmt>) {
        let ty_id = attempt!(self.resolve_type(&decl.ttype));
        let symbol = Symbol::from_global(decl, ty_id);

        let symbol = attempt!(self.register_symbol(symbol));
        let def = GlobalDef {
            name: decl.name.value().clone(),
            id: None,
            symbol,
            span: decl.name.span(),
        };
        let global_id = attempt!(self.register_global(def));
        if global_id.is_valid() {
            self.db.symbol_table.add_resolution(decl.id, symbol);
        }
    }

    fn pass_signatures(&mut self, ast: &Program) {
        for stmt in &ast.items {
            match stmt {
                Stmt::Struct(decl) => self.register_struct_fields(decl, None),
                Stmt::Function(decl) => {
                    self.register_function_decl(decl, None);
                }
                Stmt::Global(decl) => self.register_global_decl(decl),
                _ => unreachable!(),
            }
        }
    }

    pub(in crate::analyzer) fn pass_symbol_declaration(&mut self, ast: &Program) {
        self.pass_skeleton(ast);
        self.pass_signatures(ast);

        if self.db.entry_point.is_none() {
            self.report_error(
                SymbolDeclError::MissingMain.into(),
                crate::lexer::span::Span::default(),
            );
        }
    }
}
