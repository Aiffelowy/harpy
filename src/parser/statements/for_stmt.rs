use crate::lexer::tokens::Ident;
use crate::parser::parser::Parser;
use crate::parser::types::Type;
use crate::parser::{expr::Expr, parse_trait::Parse};
use crate::semantic_analyzer::analyze_trait::Analyze;
use crate::semantic_analyzer::err::SemanticError;
use crate::semantic_analyzer::scope::ScopeKind;
use crate::semantic_analyzer::symbol_info::VariableInfo;
use crate::{get_symbol, resolve_expr, t};

use super::BlockStmt;

#[derive(Debug, Clone, PartialEq)]
pub struct IterExpr {
    from: Expr,
    to: Expr,
}

impl Parse for IterExpr {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        let from = parser.parse::<Expr>()?;
        parser.consume::<t!(..)>()?;
        let to = parser.parse::<Expr>()?;

        Ok(Self { from, to })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForStmt {
    var: Ident,
    iter: IterExpr,
    block: BlockStmt,
}

impl Parse for ForStmt {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        parser.consume::<t!(for)>()?;
        let var = parser.consume::<t!(ident)>()?;
        parser.consume::<t!(in)>()?;
        let iter = parser.parse::<IterExpr>()?;
        let block = parser.parse::<BlockStmt>()?;
        Ok(Self { var, iter, block })
    }
}

impl Analyze for ForStmt {
    fn build(&self, builder: &mut crate::semantic_analyzer::scope_builder::ScopeBuilder) {
        builder.push_scope(ScopeKind::Loop);
        builder.define_var(
            &self.var,
            VariableInfo {
                ttype: Type::unknown(),
                initialized: true,
            },
        );

        self.block.build(builder);
        builder.pop_scope();
    }

    fn analyze_semantics(&self, analyzer: &mut crate::semantic_analyzer::analyzer::Analyzer) {
        analyzer.enter_scope();
        resolve_expr!(analyzer, from_type, &self.iter.from);
        resolve_expr!(analyzer, to_type, &self.iter.to);

        if from_type != Type::int() {
            analyzer.report_semantic_error(
                SemanticError::ForTypeMismatch(from_type.clone(), Type::int()),
                self.iter.to.span(),
            );
        }

        if from_type != to_type {
            analyzer.report_semantic_error(
                SemanticError::ForTypeMismatch(from_type.clone(), to_type),
                self.iter.to.span(),
            );
        }

        get_symbol!(analyzer, var, self.var);
        var.infer_type(from_type);

        self.block.analyze_semantics(analyzer);
        analyzer.exit_scope();
    }
}
