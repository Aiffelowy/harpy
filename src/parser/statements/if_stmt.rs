use crate::parser::parser::Parser;
use crate::parser::types::Type;
use crate::parser::{expr::Expr, parse_trait::Parse};
use crate::semantic_analyzer::analyze_trait::Analyze;
use crate::semantic_analyzer::err::SemanticError;
use crate::{t, tt};

use super::BlockStmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ElseStmt {
    Block(BlockStmt),
    If(Box<IfStmt>),
}

impl Parse for ElseStmt {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        parser.consume::<t!(else)>()?;
        if let tt!(if) = parser.peek()? {
            return Ok(Self::If(Box::new(parser.parse::<IfStmt>()?)));
        }

        Ok(Self::Block(parser.parse::<BlockStmt>()?))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    expr: Expr,
    block: BlockStmt,
    else_stmt: Option<ElseStmt>,
}

impl Parse for IfStmt {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        parser.consume::<t!(if)>()?;
        let expr = parser.parse::<Expr>()?;
        let block = parser.parse::<BlockStmt>()?;

        let else_stmt = if let tt!(else) = parser.peek()? {
            Some(parser.parse::<ElseStmt>()?)
        } else {
            None
        };

        Ok(Self {
            expr,
            block,
            else_stmt,
        })
    }
}

impl Analyze for IfStmt {
    fn build(&self, builder: &mut crate::semantic_analyzer::scope_builder::ScopeBuilder) {
        builder.push_scope(crate::semantic_analyzer::scope::ScopeKind::Block);
        self.block.build(builder);
        builder.pop_scope();
    }

    fn analyze_semantics(&self, analyzer: &mut crate::semantic_analyzer::analyzer::Analyzer) {
        analyzer.enter_scope();
        if let Some(expr_type) = analyzer.resolve_expr(&self.expr) {
            if expr_type != Type::bool() {
                analyzer.report_semantic_error(
                    SemanticError::IfTypeMismatch(expr_type),
                    self.expr.span(),
                );
            }
        }

        self.block.analyze_semantics(analyzer);
        analyzer.exit_scope();
    }
}
