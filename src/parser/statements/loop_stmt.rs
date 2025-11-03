use crate::parser::parse_trait::Parse;
use crate::parser::parser::Parser;
use crate::semantic_analyzer::analyze_trait::Analyze;
use crate::semantic_analyzer::scope::ScopeKind;
use crate::t;

use super::BlockStmt;

#[derive(Debug, Clone)]
pub struct LoopStmt {
    block: BlockStmt,
}

impl Parse for LoopStmt {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        parser.consume::<t!(loop)>()?;
        let block = parser.parse::<BlockStmt>()?;
        Ok(Self { block })
    }
}

impl Analyze for LoopStmt {
    fn build(&self, builder: &mut crate::semantic_analyzer::scope_builder::ScopeBuilder) {
        builder.push_scope(ScopeKind::Loop);
        self.block.build(builder);
        builder.pop_scope();
    }

    fn analyze_semantics(&self, analyzer: &mut crate::semantic_analyzer::analyzer::Analyzer) {
        analyzer.enter_scope();
        self.block.analyze_semantics(analyzer);
        analyzer.exit_scope();
    }
}
