use crate::generator::compile_trait::Generate;
use crate::generator::instruction::Instruction;
use crate::parser::node::Node;
use crate::parser::parser::Parser;
use crate::parser::types::Type;
use crate::parser::{expr::Expr, parse_trait::Parse};
use crate::semantic_analyzer::analyze_trait::Analyze;
use crate::semantic_analyzer::err::SemanticError;
use crate::semantic_analyzer::return_status::ReturnStatus;
use crate::{t, tt};

use super::BlockStmt;

#[derive(Debug, Clone)]
pub enum ElseStmt {
    Block(BlockStmt),
    If(Box<Node<IfStmt>>),
}

impl Parse for ElseStmt {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        parser.consume::<t!(else)>()?;
        if let tt!(if) = parser.peek()? {
            return Ok(Self::If(Box::new(parser.parse_node::<IfStmt>()?)));
        }

        Ok(Self::Block(parser.parse::<BlockStmt>()?))
    }
}

impl Analyze for ElseStmt {
    fn build(&self, builder: &mut crate::semantic_analyzer::scope_builder::ScopeBuilder) {
        match self {
            ElseStmt::Block(b) => b.build(builder),
            ElseStmt::If(if_stmt) => if_stmt.build(builder),
        }
    }

    fn analyze_semantics(&self, analyzer: &mut crate::semantic_analyzer::analyzer::Analyzer) -> ReturnStatus {
        match self {
            ElseStmt::Block(b) => b.analyze_semantics(analyzer),
            ElseStmt::If(if_stmt) => if_stmt.analyze_semantics(analyzer),
        }
    }
}

impl Generate for ElseStmt {
    fn generate(&self, generator: &mut crate::generator::generator::Generator) {
        match self {
            ElseStmt::Block(b) => b.generate(generator),
            ElseStmt::If(if_stmt) => if_stmt.generate(generator),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    expr: Node<Expr>,
    block: BlockStmt,
    else_stmt: Option<ElseStmt>,
}

impl Parse for IfStmt {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        parser.consume::<t!(if)>()?;
        let expr = parser.parse_node::<Expr>()?;
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
        if let Some(else_stmt) = &self.else_stmt {
            match else_stmt {
                ElseStmt::If(ifstmt) => ifstmt.build(builder),
                ElseStmt::Block(block) => block.build(builder),
            }
        }
        builder.pop_scope();
    }

    fn analyze_semantics(&self, analyzer: &mut crate::semantic_analyzer::analyzer::Analyzer) -> ReturnStatus {
        analyzer.enter_scope();
        if let Some(expr_type) = analyzer.resolve_expr(&self.expr) {
            if !expr_type.compatible(&Type::bool()) {
                analyzer.report_semantic_error(
                    SemanticError::IfTypeMismatch(expr_type),
                    self.expr.span(),
                );
            }
        }

        let then_status = self.block.analyze_semantics(analyzer);
        let else_status = if let Some(else_stmt) = &self.else_stmt {
            else_stmt.analyze_semantics(analyzer)
        } else {
            ReturnStatus::Never
        };

        analyzer.exit_scope();
        then_status.intersect(else_status)
    }
}

impl Generate for IfStmt {
    fn generate(&self, generator: &mut crate::generator::generator::Generator) {
        generator.gen_expr(&self.expr);

        if let Some(else_stmt) = &self.else_stmt {
            let else_label = generator.create_label();
            let end_label = generator.create_label();

            generator.push_instruction(Instruction::JMP_IF_FALSE(else_label));
            self.block.generate(generator);

            generator.push_instruction(Instruction::JMP(end_label));
            generator.place_label(else_label);

            else_stmt.generate(generator);
            generator.place_label(end_label);
        } else {
            let end_label = generator.create_label();
            generator.push_instruction(Instruction::JMP_IF_FALSE(end_label));
            self.block.generate(generator);

            generator.place_label(end_label);
        }
    }
}
