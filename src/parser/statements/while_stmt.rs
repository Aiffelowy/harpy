use crate::parser::node::Node;
use crate::parser::parser::Parser;
use crate::parser::types::Type;
use crate::parser::{expr::Expr, parse_trait::Parse};
use crate::semantic_analyzer::analyze_trait::Analyze;
use crate::semantic_analyzer::err::SemanticError;
use crate::semantic_analyzer::scope::ScopeKind;
use crate::t;

use super::BlockStmt;

#[derive(Debug, Clone)]
pub struct WhileStmt {
    expr: Node<Expr>,
    block: BlockStmt,
}

impl Parse for WhileStmt {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        parser.consume::<t!(while)>()?;
        let expr = parser.parse_node::<Expr>()?;
        let block = parser.parse::<BlockStmt>()?;

        Ok(Self { expr, block })
    }
}

impl Analyze for WhileStmt {
    fn build(&self, builder: &mut crate::semantic_analyzer::scope_builder::ScopeBuilder) {
        builder.push_scope(ScopeKind::Loop);
        self.block.build(builder);
        builder.pop_scope();
    }

    fn analyze_semantics(&self, analyzer: &mut crate::semantic_analyzer::analyzer::Analyzer) {
        analyzer.enter_scope();
        if let Some(expr_type) = analyzer.resolve_expr(&self.expr) {
            if !expr_type.compatible(&Type::bool()) {
                analyzer.report_semantic_error(
                    SemanticError::WhileTypeMismatch(expr_type.clone()),
                    self.expr.span(),
                );
            }
        }

        self.block.analyze_semantics(analyzer);
        analyzer.exit_scope();
    }
}
