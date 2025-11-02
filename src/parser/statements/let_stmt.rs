use crate::{
    lexer::tokens::Ident,
    parser::{expr::Expr, parse_trait::Parse, parser::Parser, types::Type},
    semantic_analyzer::{analyze_trait::Analyze, err::SemanticError, symbol_info::VariableInfo},
    t,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LetStmt {
    var: Ident,
    ttype: Type,
    rhs: Expr,
}

impl Parse for LetStmt {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        parser.consume::<t!(let)>()?;
        let var = parser.consume::<t!(ident)>()?;
        parser.consume::<t!(:)>()?;
        let ttype = parser.parse::<Type>()?;
        parser.consume::<t!(=)>()?;
        let rhs = parser.parse::<Expr>()?;
        parser.consume::<t!(;)>()?;

        Ok(Self { var, ttype, rhs })
    }
}

impl Analyze for LetStmt {
    fn build(&self, builder: &mut crate::semantic_analyzer::scope_builder::ScopeBuilder) {
        builder.define_var(
            &self.var,
            VariableInfo {
                ttype: self.ttype.clone(),
                initialized: true,
            },
        )
    }

    fn analyze_semantics(&self, analyzer: &mut crate::semantic_analyzer::analyzer::Analyzer) {
        let Some(expr_type) = analyzer.resolve_expr(&self.rhs) else {
            return;
        };

        if expr_type != self.ttype {
            analyzer.report_semantic_error(
                SemanticError::LetTypeMismatch(self.var.clone(), expr_type.clone()),
                self.rhs.span(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::Lexer, parser::parser::Parser};

    use super::LetStmt;

    #[test]
    fn test_let_stmt() {
        let mut parser = Parser::new(Lexer::new("let var: mut int = (7 + 3) * 4;").unwrap());
        parser.parse::<LetStmt>().unwrap();
    }
}
