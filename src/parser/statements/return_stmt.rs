use std::rc::Rc;

use crate::extensions::SymbolInfoRefExt;
use crate::lexer::span::Span;
use crate::parser::node::Node;
use crate::parser::parser::Parser;
use crate::parser::types::Type;
use crate::parser::{expr::Expr, parse_trait::Parse};
use crate::semantic_analyzer::analyze_trait::Analyze;
use crate::semantic_analyzer::err::SemanticError;
use crate::semantic_analyzer::type_table::TypeIndex;
use crate::{t, tt};

#[derive(Debug, Clone)]
pub struct ReturnStmt {
    span: Span,
    expr: Option<Node<Expr>>,
}

impl Parse for ReturnStmt {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        let span = parser.consume::<t!(return)>()?.span();

        if let tt!(;) = parser.peek()? {
            parser.consume::<t!(;)>()?;
            return Ok(Self { expr: None, span });
        }

        let expr = parser.parse_node::<Expr>()?;
        parser.consume::<t!(;)>()?;
        Ok(Self {
            expr: Some(expr),
            span,
        })
    }
}

impl Analyze for ReturnStmt {
    fn build(&self, _builder: &mut crate::semantic_analyzer::scope_builder::ScopeBuilder) {}
    fn analyze_semantics(&self, analyzer: &mut crate::semantic_analyzer::analyzer::Analyzer) {
        let Some(rt) = analyzer.get_func_info() else {
            analyzer.report_semantic_error(SemanticError::ReturnNotInFunc, self.span);
            return;
        };

        let rt = rt.get().ty.clone();

        let Some(ref expr) = self.expr else {
            if !rt.compatible(&Type::void()) {
                analyzer.report_semantic_error(
                    SemanticError::ReturnTypeMismatch(
                        Rc::new(crate::semantic_analyzer::symbol_info::TypeInfo {
                            ttype: Type::void(),
                            size: 0,
                            idx: TypeIndex(0),
                        }),
                        rt.clone(),
                    ),
                    self.span,
                );
            }

            return;
        };

        if let Some(expr_type) = analyzer.resolve_expr(expr) {
            if !expr_type.return_compatible(&rt.ttype) {
                analyzer.report_semantic_error(
                    SemanticError::ReturnTypeMismatch(expr_type.clone(), rt.clone()),
                    expr.span(),
                );
            }

            if let Some(i) = expr.lvalue() {
                analyzer.check_return_borrow(i);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::Lexer, parser::parser::Parser};

    use super::ReturnStmt;

    #[test]
    fn test_return_stmt() {
        let mut parser = Parser::new(Lexer::new("return a == b").unwrap());
        parser.parse::<ReturnStmt>().unwrap();
    }
}
