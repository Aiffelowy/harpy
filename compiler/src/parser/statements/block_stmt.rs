use crate::generator::compile_trait::Generate;
use crate::parser::parser::Parser;
use crate::parser::{parse_trait::Parse, statements::Stmt};
use crate::semantic_analyzer::analyze_trait::Analyze;
use crate::semantic_analyzer::return_status::ReturnStatus;
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

    fn analyze_semantics(&self, analyzer: &mut crate::semantic_analyzer::analyzer::Analyzer) -> ReturnStatus {
        analyzer.enter_scope();
        
        let mut status = ReturnStatus::Never;
        for stmt in &self.stmts {
            let stmt_status = stmt.analyze_semantics(analyzer);
            status = status.then(stmt_status);
        }
        
        analyzer.exit_scope();
        status
    }
}

impl Generate for BlockStmt {
    fn generate(&self, generator: &mut crate::generator::generator::Generator) {
        for stmt in &self.stmts {
            stmt.generate(generator);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BlockStmt;
    use crate::{lexer::Lexer, parser::{parser::Parser, Parse}, source::SourceFile};
    use std::io::Cursor;

    fn parse_block(input: &str) -> BlockStmt {
        let source = SourceFile::new(Cursor::new(input)).unwrap();
        let mut parser = Parser::new(Lexer::new(&source).unwrap());
        BlockStmt::parse(&mut parser).unwrap()
    }

    #[test]
    fn test_block_stmt_empty() {
        let block = parse_block("{}");
        assert_eq!(block.stmts.len(), 0);
    }

    #[test]
    fn test_block_stmt_single_statement() {
        let block = parse_block("{ let x = 5; }");
        assert_eq!(block.stmts.len(), 1);
    }

    #[test]
    fn test_block_stmt_multiple_statements() {
        let block = parse_block("{ let x = 5; let y = 10; return x + y; }");
        assert_eq!(block.stmts.len(), 3);
    }

    #[test]
    fn test_block_stmt_nested() {
        let block = parse_block("{ { let x = 1; let y = 2; } }");
        assert_eq!(block.stmts.len(), 1);
    }
}
