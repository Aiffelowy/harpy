use crate::generator::compile_trait::Generate;
use crate::generator::instruction::Instruction;
use crate::lexer::tokens::Ident;
use crate::parser::node::Node;
use crate::parser::parser::Parser;
use crate::parser::types::Type;
use crate::parser::{expr::Expr, parse_trait::Parse};
use crate::semantic_analyzer::analyze_trait::Analyze;
use crate::semantic_analyzer::return_status::ReturnStatus;
use crate::semantic_analyzer::err::SemanticError;
use crate::semantic_analyzer::scope::ScopeKind;
use crate::semantic_analyzer::symbol_info::SymbolInfoKind;
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

    fn analyze_semantics(&self, analyzer: &mut crate::semantic_analyzer::analyzer::Analyzer) -> ReturnStatus {
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
                    if let SymbolInfoKind::Variable(ref mut v) = var.kind {
                        v.initialized = true;
                    }
                });
            }
        }

        let block_status = self.block.analyze_semantics(analyzer);
        analyzer.exit_scope();
        match block_status {
            ReturnStatus::Always => ReturnStatus::Sometimes,
            _ => ReturnStatus::Never,
        }
    }
}

impl Generate for ForStmt {
    fn generate(&self, generator: &mut crate::generator::generator::Generator) {
        generator.gen_expr(&self.iter.from);
        let iter_var = generator.get_local_mapping(self.var.id());
        generator.push_instruction(Instruction::STORE_LOCAL(iter_var));

        let loop_start = generator.create_label();
        generator.place_label(loop_start);

        generator.push_instruction(Instruction::LOAD_LOCAL(iter_var));
        generator.gen_expr(&self.iter.to);
        generator.push_instruction(Instruction::LT);

        let loop_end = generator.create_label();
        generator.push_instruction(Instruction::JMP_IF_FALSE(loop_end));

        self.block.generate(generator);

        generator.push_instruction(Instruction::LOAD_LOCAL(iter_var));
        generator.push_instruction(Instruction::INC);
        generator.push_instruction(Instruction::STORE_LOCAL(iter_var));

        generator.push_instruction(Instruction::JMP(loop_start));

        generator.place_label(loop_end);
    }
}

#[cfg(test)]
mod tests {
    use super::ForStmt;
    use crate::{lexer::Lexer, parser::{parser::Parser, Parse}, source::SourceFile};
    use std::io::Cursor;

    fn parse_for(input: &str) -> ForStmt {
        let source = SourceFile::new(Cursor::new(input)).unwrap();
        let mut parser = Parser::new(Lexer::new(&source).unwrap());
        ForStmt::parse(&mut parser).unwrap()
    }

    #[test]
    fn test_for_stmt_basic() {
        let for_stmt = parse_for("for i in 0 => 10 { }");
        assert_eq!(for_stmt.var.value(), "i");
    }

    #[test]
    fn test_for_stmt_arithmetic() {
        let for_stmt = parse_for("for i in 5+2 => 50%4 { }");
        assert_eq!(for_stmt.var.value(), "i");
    }

    #[test]
    fn test_for_stmt_with_body() {
        let for_stmt = parse_for("for i in 1 => 5 { let x = i; }");
        assert_eq!(for_stmt.var.value(), "i");
    }

    #[test]
    fn test_for_stmt_complex_range() {
        let for_stmt = parse_for("for j in start => end { return j; }");
        assert_eq!(for_stmt.var.value(), "j");
    }
}
