use crate::generator::compile_trait::Generate;
use crate::generator::instruction::Instruction;
use crate::lexer::tokens::Lit;
use crate::parser::node::Node;
use crate::parser::parser::Parser;
use crate::parser::types::Type;
use crate::parser::{expr::Expr, parse_trait::Parse};
use crate::semantic_analyzer::analyze_trait::Analyze;
use crate::semantic_analyzer::err::SemanticError;
use crate::semantic_analyzer::return_status::ReturnStatus;
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

    fn analyze_semantics(
        &self,
        analyzer: &mut crate::semantic_analyzer::analyzer::Analyzer,
    ) -> ReturnStatus {
        analyzer.enter_scope();
        if let Some(expr_type) = analyzer.resolve_expr(&self.expr) {
            if !expr_type.compatible(&Type::bool()) {
                analyzer.report_semantic_error(
                    SemanticError::WhileTypeMismatch(expr_type.clone()),
                    self.expr.span(),
                );
            }
        }

        let block_status = self.block.analyze_semantics(analyzer);
        analyzer.exit_scope();

        let is_infinite_loop = match &*self.expr {
            Expr::Literal(lit) => matches!(lit.value(), Lit::LitBool(true)),
            _ => false,
        };

        if is_infinite_loop {
            block_status
        } else {
            match block_status {
                ReturnStatus::Always => ReturnStatus::Sometimes,
                _ => ReturnStatus::Never,
            }
        }
    }
}

impl Generate for WhileStmt {
    fn generate(&self, generator: &mut crate::generator::generator::Generator) {
        let loop_start = generator.create_label();
        generator.place_label(loop_start);
        generator.gen_expr(&self.expr);
        let loop_end = generator.create_label();
        generator.push_instruction(crate::generator::instruction::Instruction::JMP_IF_FALSE(
            loop_end,
        ));
        self.block.generate(generator);
        generator.push_instruction(Instruction::JMP(loop_start));
        generator.place_label(loop_end);
    }
}

#[cfg(test)]
mod tests {
    use super::WhileStmt;
    use crate::{lexer::Lexer, parser::{parser::Parser, Parse}, source::SourceFile};
    use std::io::Cursor;

    fn parse_while(input: &str) -> WhileStmt {
        let source = SourceFile::new(Cursor::new(input)).unwrap();
        let mut parser = Parser::new(Lexer::new(&source).unwrap());
        WhileStmt::parse(&mut parser).unwrap()
    }

    #[test]
    fn test_while_stmt_simple() {
        parse_while("while true { }");
    }

    #[test]
    fn test_while_stmt_with_body() {
        parse_while("while x < 10 { x += 1; }");
    }

    #[test]
    fn test_while_stmt_complex_condition() {
        parse_while("while x > 0 && y != 5 { let z = x * y; x -= 1; }");
    }
}
