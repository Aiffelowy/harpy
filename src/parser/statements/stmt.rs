use crate::{
    get_symbol_mut,
    parser::{expr::Expr, node::Node, parser::Parser, Parse},
    semantic_analyzer::{analyze_trait::Analyze, err::SemanticError, symbol_info::SymbolInfoKind},
    t, tt,
};

use super::{
    assign_stmt::AssignOp, BlockStmt, ForStmt, IfStmt, LetStmt, LoopStmt, ReturnStmt, WhileStmt,
};

#[derive(Debug, Clone)]
pub enum Stmt {
    LetStmt(Node<LetStmt>),
    IfStmt(Node<IfStmt>),
    ForStmt(Node<ForStmt>),
    WhileStmt(Node<WhileStmt>),
    LoopStmt(Node<LoopStmt>),
    ReturnStmt(Node<ReturnStmt>),
    AssignStmt(Node<Expr>, AssignOp, Node<Expr>),
    BlockStmt(Node<BlockStmt>),
    Expr(Node<Expr>),
}

impl Parse for Stmt {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        let s = match parser.peek()? {
            tt!("{") => Self::BlockStmt(parser.parse_node::<BlockStmt>()?),
            tt!(let) => Self::LetStmt(parser.parse_node::<LetStmt>()?),
            tt!(if) => Self::IfStmt(parser.parse_node::<IfStmt>()?),
            tt!(for) => Self::ForStmt(parser.parse_node::<ForStmt>()?),
            tt!(while) => Self::WhileStmt(parser.parse_node::<WhileStmt>()?),
            tt!(loop) => Self::LoopStmt(parser.parse_node::<LoopStmt>()?),
            tt!(return) => Self::ReturnStmt(parser.parse_node::<ReturnStmt>()?),
            _ => {
                let expr = parser.parse_node::<Expr>()?;
                if let Some(assign) = parser.try_parse::<AssignOp>() {
                    let s = Self::AssignStmt(expr, assign, parser.parse_node::<Expr>()?);
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
                let Some(lhs_type) = analyzer.resolve_expr_write(lhs) else {
                    return;
                };
                let Some(rhs_type) = analyzer.resolve_expr(rhs) else {
                    return;
                };

                let mut lhs_type = lhs_type.deref();

                if let Some(i) = lhs.lvalue() {
                    get_symbol_mut!((analyzer, i) info {
                        if let SymbolInfoKind::Variable(ref mut v) = info.kind {
                            if !lhs_type.mutable && v.initialized {
                                analyzer.report_semantic_error(
                                    SemanticError::AssignToConst(lhs.clone()),
                                    lhs.span(),
                                );
                            }

                            if !v.initialized {
                                v.initialized = true;
                                info.infer_type(&rhs_type);
                                lhs_type = &rhs_type.ttype;
                            }

                            if !lhs_type.assign_compatible(&rhs_type.ttype) {
                                analyzer.report_semantic_error(
                                    SemanticError::AssignTypeMismatch(rhs_type.clone(), lhs_type.clone()),
                                    rhs.span(),
                                );
                            }
                        }
                    });
                } else {
                    analyzer.report_semantic_error(SemanticError::AssignToRValue, lhs.span());
                    return;
                };
            }
        }
    }
}
