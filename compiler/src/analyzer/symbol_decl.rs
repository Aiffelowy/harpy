use crate::{
    analyzer::{
        analyzer::Analyzer,
        err::SymbolDeclError,
        modules::ModuleId,
        tables::{
            function_table::FunctionDef,
            global_table::GlobalDef,
            struct_table::{Field, StructLayout},
            symbol_table::Symbol,
        },
    },
    attempt,
    parser::{
        stmt::stmts::{FunctionDecl, GlobalStmt, Program, Stmt, StructDecl},
        types::type_parsing::FunctionType,
    },
};

impl Analyzer {
    fn pass_skeleton(&mut self, ast: &Program) {
        for stmt in &ast.items {
            if let Stmt::Struct(decl) = &stmt {
                let layout = StructLayout::skeleton(decl.name.value().clone(), decl.name.span());
                attempt!(self, self.register_struct(layout));
            }
        }
    }

    fn register_struct_fields(&mut self, decl: &StructDecl) {
        let mut fields = vec![];
        for field in &decl.fields {
            let ty_id = attempt!(self, self.resolve_type(&field.inner.ttype));
            fields.push(Field {
                name: field.inner.name.value().clone(),
                ty: ty_id,
            });
        }

        let struct_id = attempt!(self, self.resolve_struct_name(&decl.name));
        if !struct_id.is_valid() {
            return;
        }
        self.db.struct_table.get_mut(struct_id).set_fields(fields);
    }

    fn register_function_decl(&mut self, decl: &FunctionDecl) {
        let args = decl.args.iter().map(|arg| arg.ty.clone()).collect();
        let fn_ty = FunctionType {
            args,
            return_type: Box::new(decl.return_type.clone()),
        };
        let signature = attempt!(self, self.resolve_fn_type(&fn_ty));
        if !signature.is_valid() {
            return;
        }
        let def = FunctionDef::skeleton(decl.name.value().clone(), signature, decl.name.span());
        let func_id = attempt!(self, self.register_function(def));

        if decl.name.value() == "main" {
            self.db.entry_point = Some(func_id)
        }
    }

    fn register_global_decl(&mut self, decl: &GlobalStmt) {
        let ty_id = attempt!(self, self.resolve_type(&decl.ttype));
        let symbol = Symbol {
            id: None,
            name: decl.name.value().clone(),
            ty: ty_id,
            is_mutable: decl.mutable.0,
            declared_at: decl.name.span(),
        };

        let symbol = attempt!(self, self.register_symbol(symbol));
        let def = GlobalDef {
            name: decl.name.value().clone(),
            id: None,
            symbol,
            span: decl.name.span(),
        };
        attempt!(self, self.register_global(def));
    }

    fn pass_signatures(&mut self, ast: &Program) {
        for stmt in &ast.items {
            match stmt {
                Stmt::Struct(decl) => self.register_struct_fields(decl),
                Stmt::Function(decl) => self.register_function_decl(decl),
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
