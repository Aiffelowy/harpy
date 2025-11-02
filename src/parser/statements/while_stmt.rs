use crate::parser::parser::Parser;
use crate::parser::types::Type;
use crate::parser::{expr::Expr, parse_trait::Parse};
use crate::semantic_analyzer::analyze_trait::Analyze;
use crate::semantic_analyzer::err::SemanticError;
use crate::semantic_analyzer::scope::ScopeKind;
use crate::{resolve_expr, t};

use super::BlockStmt;

#[derive(Debug, Clone, PartialEq)]
pub struct WhileStmt {
    expr: Expr,
    block: BlockStmt,
}

impl Parse for WhileStmt {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        parser.consume::<t!(while)>()?;
        let expr = parser.parse::<Expr>()?;
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
        resolve_expr!(analyzer, expr_type, &self.expr);

        if expr_type != Type::bool() {
            analyzer.report_semantic_error(
                SemanticError::WhileTypeMismatch(expr_type),
                self.expr.span(),
            );
        }

        self.block.analyze_semantics(analyzer);
        analyzer.exit_scope();
    }
}
