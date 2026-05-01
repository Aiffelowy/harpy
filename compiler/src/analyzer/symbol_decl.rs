use crate::{
    aliases::Result,
    analyzer::{
        analyzer::Analyzer,
        modules::ModuleId,
        tables::{
            function_table::FunctionDef,
            global_table::GlobalDef,
            struct_table::{Field, StructLayout},
            symbol_table::Symbol,
        },
    },
    parser::{
        stmt::stmts::{FunctionDecl, GlobalStmt, Program, Stmt, StructDecl},
        types::type_parsing::FunctionType,
    },
};

impl Analyzer {
    fn pass_skeleton(&mut self, ast: &Program, module_id: ModuleId) -> Result<()> {
        for stmt in &ast.items {
            if let Stmt::Struct(decl) = &stmt {
                let layout = StructLayout::skeleton(decl.name.value().clone(), decl.name.span());
                self.register_struct(module_id, layout)?;
            }
        }

        Ok(())
    }

    fn register_struct_fields(&mut self, decl: &StructDecl, module_id: ModuleId) -> Result<()> {
        let mut fields = vec![];
        for field in &decl.fields {
            let ty_id = self.resolve_type(&field.inner.ttype, module_id)?;
            fields.push(Field {
                name: field.inner.name.value().clone(),
                ty: ty_id,
            });
        }

        let struct_id = self.resolve_struct_name(module_id, &decl.name)?;
        self.db.struct_table.get_mut(struct_id).set_fields(fields);
        Ok(())
    }

    fn register_function_decl(&mut self, decl: &FunctionDecl, module_id: ModuleId) -> Result<()> {
        let args = decl.args.iter().map(|arg| arg.ty.clone()).collect();
        let fn_ty = FunctionType {
            args,
            return_type: Box::new(decl.return_type.clone()),
        };
        let signature = self.resolve_fn_type(module_id, &fn_ty)?;
        let def = FunctionDef::skeleton(decl.name.value().clone(), signature, decl.name.span());
        self.register_function(module_id, def)?;
        Ok(())
    }

    fn register_global_decl(&mut self, decl: &GlobalStmt, module_id: ModuleId) -> Result<()> {
        let ty_id = self.resolve_type(&decl.ttype, module_id)?;
        let symbol = Symbol {
            id: None,
            name: decl.name.value().clone(),
            ty: ty_id,
            is_mutable: decl.mutable.0,
            declared_at: decl.name.span(),
        };
        let symbol = self.register_symbol(symbol)?;
        let def = GlobalDef {
            name: decl.name.value().clone(),
            id: None,
            symbol,
            span: decl.name.span(),
        };
        self.register_global(module_id, def)?;
        Ok(())
    }

    fn pass_signatures(&mut self, ast: &Program, module_id: ModuleId) -> Result<()> {
        for stmt in &ast.items {
            match stmt {
                Stmt::Struct(decl) => self.register_struct_fields(decl, module_id)?,
                Stmt::Function(decl) => self.register_function_decl(decl, module_id)?,
                Stmt::Global(decl) => self.register_global_decl(decl, module_id)?,
                _ => unreachable!(),
            }
        }
        Ok(())
    }

    pub(in crate::analyzer) fn pass_symbol_declaration(
        &mut self,
        ast: &Program,
        module_id: ModuleId,
    ) -> Result<()> {
        self.pass_skeleton(ast, module_id)?;
        self.pass_signatures(ast, module_id)?;
        Ok(())
    }
}
