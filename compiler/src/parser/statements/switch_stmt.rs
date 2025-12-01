use crate::generator::compile_trait::Generate;
use crate::generator::instruction::Instruction;
use crate::parser::{expr::Expr, node::Node, Parse};
use crate::semantic_analyzer::analyze_trait::Analyze;
use crate::semantic_analyzer::err::SemanticError;
use crate::semantic_analyzer::return_status::ReturnStatus;
use crate::{t, tt};

use super::Stmt;

#[derive(Debug, Clone)]
pub struct CaseStmt {
    expr: Option<Node<Expr>>,
    stmt: Stmt,
}

impl Parse for CaseStmt {
    fn parse(parser: &mut crate::parser::parser::Parser) -> crate::aliases::Result<Self> {
        let expr = if let tt!(.) = parser.peek()? {
            parser.consume::<t!(.)>()?;
            None
        } else {
            Some(parser.parse_node()?)
        };

        parser.consume::<t!(->)>()?;
        let stmt = parser.parse()?;
        //parser.consume::<t!(,)>()?;

        Ok(Self { expr, stmt })
    }
}

#[derive(Debug, Clone)]
pub struct SwitchStmt {
    expr: Node<Expr>,
    cases: Vec<CaseStmt>,
}

impl Parse for SwitchStmt {
    fn parse(parser: &mut crate::parser::parser::Parser) -> crate::aliases::Result<Self> {
        parser.consume::<t!(switch)>()?;
        let expr = parser.parse_node()?;
        let mut cases = vec![];
        parser.consume::<t!("{")>()?;
        loop {
            if let tt!("}") | tt!(eof) = parser.peek()? {
                break;
            }
            match parser.parse::<CaseStmt>() {
                Ok(case) => cases.push(case),
                Err(e) => parser.report_error(e, &[tt!(,)])?,
            }
        }

        parser.consume::<t!("}")>()?;
        Ok(Self { expr, cases })
    }
}

impl Analyze for SwitchStmt {
    fn build(&self, builder: &mut crate::semantic_analyzer::scope_builder::ScopeBuilder) {
        for case in &self.cases {
            builder.push_scope(crate::semantic_analyzer::scope::ScopeKind::Block);
            case.stmt.build(builder);
            builder.pop_scope();
        }
    }

    fn analyze_semantics(
        &self,
        analyzer: &mut crate::semantic_analyzer::analyzer::Analyzer,
    ) -> crate::semantic_analyzer::return_status::ReturnStatus {
        let Some(main_expr_ty) = analyzer.resolve_expr(&self.expr) else {
            analyzer.report_semantic_error(SemanticError::UnresolvedType, self.expr.span());
            return ReturnStatus::Never;
        };

        let has_default = self.cases.iter().any(|case| case.expr.is_none());
        let mut return_status = if has_default { ReturnStatus::Always } else { ReturnStatus::Never };

        for case in &self.cases {
            analyzer.enter_scope();
            if let Some(case_expr) = &case.expr {
                let Some(case_expr_ty) = analyzer.resolve_expr(&case_expr) else {
                    analyzer.report_semantic_error(SemanticError::UnresolvedType, case_expr.span());
                    analyzer.exit_scope();
                    continue;
                };

                if !case_expr_ty.compatible(&main_expr_ty) {
                    analyzer.report_semantic_error(
                        SemanticError::SwitchTypeMismatch(case_expr_ty, main_expr_ty.clone()),
                        case_expr.span(),
                    );
                }
            }

            let status = case.stmt.analyze_semantics(analyzer);
            return_status = return_status.intersect(status);
            analyzer.exit_scope();
        }

        return_status
    }
}

impl Generate for SwitchStmt {
    fn generate(&self, generator: &mut crate::generator::generator::Generator) {
        generator.gen_expr(&self.expr);

        let end_label = generator.create_label();
        let case_labels: Vec<_> = self
            .cases
            .iter()
            .map(|_| generator.create_label())
            .collect();

        for (i, case) in self.cases.iter().enumerate() {
            let Some(case_expr) = &case.expr else {
                generator.push_instruction(Instruction::JMP(case_labels[i]));
                continue;
            };
            generator.push_instruction(Instruction::DUP);
            generator.gen_expr(case_expr);
            generator.push_instruction(Instruction::EQ);
            generator.push_instruction(Instruction::JMP_IF_TRUE(
                case_labels[i],
            ));
        }

        generator.push_instruction(Instruction::POP);
        generator.push_instruction(Instruction::JMP(end_label));

        for (i, case) in self.cases.iter().enumerate() {
            generator.place_label(case_labels[i]);
            generator.push_instruction(Instruction::POP);
            case.stmt.generate(generator);
            generator.push_instruction(Instruction::JMP(end_label));
        }

        generator.place_label(end_label);
    }
}

#[cfg(test)]
mod tests {
    use super::SwitchStmt;
    use crate::{lexer::Lexer, parser::parser::Parser, source::SourceFile};
    use std::io::Cursor;

    #[test]
    fn test_switch_stmt() {
        let source =
            SourceFile::new(Cursor::new("switch x { 1 -> return 1; 2 -> return 2; }")).unwrap();
        let mut parser = Parser::new(Lexer::new(&source).unwrap());
        parser.parse::<SwitchStmt>().unwrap();
    }

    #[test]
    fn test_switch_stmt_empty() {
        let source = SourceFile::new(Cursor::new("switch x {}")).unwrap();
        let mut parser = Parser::new(Lexer::new(&source).unwrap());
        parser.parse::<SwitchStmt>().unwrap();
    }

    #[test]
    fn test_switch_stmt_single_case() {
        let source = SourceFile::new(Cursor::new("switch x { 42 -> { let y = x; } }")).unwrap();
        let mut parser = Parser::new(Lexer::new(&source).unwrap());
        parser.parse::<SwitchStmt>().unwrap();
    }
}
