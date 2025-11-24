use crate::{
    generator::{compile_trait::Generate, instruction::Instruction},
    get_symbol_mut,
    lexer::tokens::Ident,
    parser::{expr::Expr, node::Node, parse_trait::Parse, parser::Parser, types::TypeSpanned},
    semantic_analyzer::{
        analyze_trait::Analyze, err::SemanticError, return_status::ReturnStatus,
        symbol_info::SymbolInfoKind,
    },
    t,
};

#[derive(Debug, Clone)]
pub struct GlobalStmt {
    pub var: Node<Ident>,
    pub ttype: TypeSpanned,
    pub rhs: Node<Expr>,
}

impl Parse for GlobalStmt {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        parser.consume::<t!(global)>()?;
        let var = parser.parse_node::<Ident>()?;

        parser.consume::<t!(:)>()?;
        let ttype = parser.parse()?;

        parser.consume::<t!(=)>()?;
        let rhs = parser.parse_node::<Expr>()?;

        parser.consume::<t!(;)>()?;

        Ok(Self { var, ttype, rhs })
    }
}

impl Analyze for GlobalStmt {
    fn build(&self, builder: &mut crate::semantic_analyzer::scope_builder::ScopeBuilder) {
        let type_info = builder.register_type(&self.ttype);
        builder.define_global(&self.var, type_info);
    }

    fn analyze_semantics(
        &self,
        analyzer: &mut crate::semantic_analyzer::analyzer::Analyzer,
    ) -> ReturnStatus {
        let Some(expr_type) = analyzer.resolve_expr(&self.rhs) else {
            return ReturnStatus::Never;
        };

        get_symbol_mut!((analyzer, self.var) info {
            if !info.ty.assign_compatible(&expr_type.ttype) {
                analyzer.report_semantic_error(
                    SemanticError::LetTypeMismatch(info.ty.ttype.clone(), expr_type.clone()),
                    self.rhs.span(),
                );
            }

            if let SymbolInfoKind::Variable(ref mut v) = &mut info.kind {
                v.initialized = true;
            }
        });

        ReturnStatus::Never
    }
}

impl Generate for GlobalStmt {
    fn generate(&self, generator: &mut crate::generator::generator::Generator) {
        generator.gen_expr(&self.rhs);
        let id = generator.get_global_mapping(self.var.id());
        generator.push_instruction(Instruction::STORE_GLOBAL(id));
    }
}
