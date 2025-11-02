use crate::{
    parser::{expr::Expr, parser::Parser, Parse},
    resolve_expr,
    semantic_analyzer::{analyze_trait::Analyze, err::SemanticError},
    t, tt,
};

use super::{
    assign_stmt::AssignOp, BlockStmt, ForStmt, IfStmt, LetStmt, LoopStmt, ReturnStmt, WhileStmt,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    LetStmt(LetStmt),
    IfStmt(IfStmt),
    ForStmt(ForStmt),
    WhileStmt(WhileStmt),
    LoopStmt(LoopStmt),
    ReturnStmt(ReturnStmt),
    AssignStmt(Expr, AssignOp, Expr),
    BlockStmt(BlockStmt),
    Expr(Expr),
}

impl Parse for Stmt {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        let s = match parser.peek()? {
            tt!("{") => Self::BlockStmt(BlockStmt::parse(parser)?),
            tt!(let) => Self::LetStmt(LetStmt::parse(parser)?),
            tt!(if) => Self::IfStmt(IfStmt::parse(parser)?),
            tt!(for) => Self::ForStmt(ForStmt::parse(parser)?),
            tt!(while) => Self::WhileStmt(WhileStmt::parse(parser)?),
            tt!(loop) => Self::LoopStmt(LoopStmt::parse(parser)?),
            tt!(return) => Self::ReturnStmt(ReturnStmt::parse(parser)?),
            _ => {
                let expr = parser.parse::<Expr>()?;
                if let Some(assign) = parser.try_parse::<AssignOp>() {
                    let s = Self::AssignStmt(expr, assign, parser.parse::<Expr>()?);
                    parser.consume::<t!(;)>()?;
                    s
                } else {
                    parser.consume::<t!(;)>()?;
                    Self::Expr(expr)
                }
            }
        };

        Ok(s)
    }
}

impl Analyze for Stmt {
    fn build(&self, builder: &mut crate::semantic_analyzer::scope_builder::ScopeBuilder) {
        use Stmt::*;
        match self {
            LetStmt(lets) => lets.build(builder),
            BlockStmt(block) => block.build(builder),
            IfStmt(ifs) => ifs.build(builder),
            ForStmt(fors) => fors.build(builder),
            WhileStmt(whiles) => whiles.build(builder),
            LoopStmt(loops) => loops.build(builder),
            ReturnStmt(returns) => returns.build(builder),
            _ => (),
        }
    }

    fn analyze_semantics(&self, analyzer: &mut crate::semantic_analyzer::analyzer::Analyzer) {
        use Stmt::*;
        match self {
            LetStmt(lets) => lets.analyze_semantics(analyzer),
            BlockStmt(block) => block.analyze_semantics(analyzer),
            IfStmt(ifs) => ifs.analyze_semantics(analyzer),
            ForStmt(fors) => fors.analyze_semantics(analyzer),
            WhileStmt(whiles) => whiles.analyze_semantics(analyzer),
            LoopStmt(loops) => loops.analyze_semantics(analyzer),
            ReturnStmt(returns) => returns.analyze_semantics(analyzer),
            Expr(expr) => {
                if let None = analyzer.resolve_expr(expr) {
                    return;
                }
            }
            AssignStmt(lhs, _, rhs) => {
                resolve_expr!(analyzer, lhs_type, lhs);
                resolve_expr!(analyzer, rhs_type, rhs);

                if !lhs_type.mutable {
                    analyzer.report_semantic_error(
                        SemanticError::AssignToConst(lhs.clone()),
                        lhs.span(),
                    );
                }

                if lhs_type != rhs_type {
                    analyzer.report_semantic_error(
                        SemanticError::AssignTypeMismatch(rhs_type, lhs_type),
                        rhs.span(),
                    );
                }
            }
        }
    }
}
