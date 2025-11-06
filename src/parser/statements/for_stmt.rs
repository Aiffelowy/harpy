use crate::lexer::tokens::Ident;
use crate::parser::node::Node;
use crate::parser::parser::Parser;
use crate::parser::types::Type;
use crate::parser::{expr::Expr, parse_trait::Parse};
use crate::semantic_analyzer::analyze_trait::Analyze;
use crate::semantic_analyzer::err::SemanticError;
use crate::semantic_analyzer::scope::ScopeKind;
use crate::{get_symbol_mut, t};

use super::BlockStmt;

#[derive(Debug, Clone)]
pub struct IterExpr {
    from: Node<Expr>,
    to: Node<Expr>,
}

impl Parse for IterExpr {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        let from = parser.parse_node::<Expr>()?;
        parser.consume::<t!(=>)>()?;
        let to = parser.parse_node::<Expr>()?;

        Ok(Self { from, to })
    }
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    var: Node<Ident>,
    iter: IterExpr,
    block: BlockStmt,
}

impl Parse for ForStmt {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        parser.consume::<t!(for)>()?;
        let var = parser.parse_node()?;
        parser.consume::<t!(in)>()?;
        let iter = parser.parse()?;
        let block = parser.parse()?;
        Ok(Self { var, iter, block })
    }
}

impl Analyze for ForStmt {
    fn build(&self, builder: &mut crate::semantic_analyzer::scope_builder::ScopeBuilder) {
        builder.push_scope(ScopeKind::Loop);

        let type_info =
            builder.register_type(&crate::parser::types::TypeSpanned::dummy(Type::unknown()));

        builder.define_var(&self.var, type_info);

        self.block.build(builder);
        builder.pop_scope();
    }

    fn analyze_semantics(&self, analyzer: &mut crate::semantic_analyzer::analyzer::Analyzer) {
        analyzer.enter_scope();

        if let Some(from_type) = analyzer.resolve_expr(&self.iter.from) {
            if !from_type.compatible(&Type::int()) {
                analyzer.report_semantic_error(
                    SemanticError::ForTypeMismatch(from_type.clone(), Type::int()),
                    self.iter.to.span(),
                );
            }

            if let Some(to_type) = analyzer.resolve_expr(&self.iter.to) {
                if !to_type.compatible(&from_type.ttype) {
                    analyzer.report_semantic_error(
                        SemanticError::ForTypeMismatch(from_type.clone(), to_type.ttype.clone()),
                        self.iter.to.span(),
                    );
                }

                get_symbol_mut!((analyzer, self.var) var {
                    var.infer_type(&from_type);
                });
            }
        }

        self.block.analyze_semantics(analyzer);
        analyzer.exit_scope();
    }
}
