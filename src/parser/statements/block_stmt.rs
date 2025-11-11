use crate::generator::compile_trait::Generate;
use crate::parser::parser::Parser;
use crate::parser::{parse_trait::Parse, statements::Stmt};
use crate::semantic_analyzer::analyze_trait::Analyze;
use crate::semantic_analyzer::scope::ScopeKind;
use crate::{t, tt};

#[derive(Debug, Clone)]
pub struct BlockStmt {
    stmts: Vec<Stmt>,
}

impl Parse for BlockStmt {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        let mut stmts = vec![];
        parser.consume::<t!("{")>()?;

        loop {
            if let tt!("}") | tt!(eof) = parser.peek()? {
                break;
            }

            match parser.parse::<Stmt>() {
                Ok(s) => stmts.push(s),
                Err(e) => parser.report_error(e, &[])?,
            }
        }

        parser.consume::<t!("}")>()?;
        Ok(Self { stmts })
    }
}

impl Analyze for BlockStmt {
    fn build(&self, builder: &mut crate::semantic_analyzer::scope_builder::ScopeBuilder) {
        builder.push_scope(ScopeKind::Block);
        for stmt in &self.stmts {
            stmt.build(builder)
        }
        builder.pop_scope();
    }

    fn analyze_semantics(&self, analyzer: &mut crate::semantic_analyzer::analyzer::Analyzer) {
        analyzer.enter_scope();
        for stmt in &self.stmts {
            stmt.analyze_semantics(analyzer)
        }
        analyzer.exit_scope();
    }
}

impl Generate for BlockStmt {
    fn generate(&self, generator: &mut crate::generator::generator::Generator) {
        for stmt in &self.stmts {
            stmt.generate(generator);
        }
    }
}
