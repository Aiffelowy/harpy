use crate::{
    get_symbol_mut,
    lexer::tokens::Ident,
    parser::{
        expr::Expr,
        node::Node,
        parse_trait::Parse,
        parser::Parser,
        types::{Type, TypeInner, TypeSpanned},
    },
    semantic_analyzer::{analyze_trait::Analyze, err::SemanticError},
    t, tt,
};

#[derive(Debug, Clone)]
pub struct LetStmt {
    var: Node<Ident>,
    ttype: TypeSpanned,
    rhs: Node<Expr>,
}

impl Parse for LetStmt {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        parser.consume::<t!(let)>()?;
        let var = parser.parse_node::<Ident>()?;

        let mut ttype = TypeSpanned::dummy(Type::unknown());
        if let tt!(:) = parser.peek()? {
            parser.consume::<t!(:)>()?;
            ttype = parser.parse()?;
        }

        parser.consume::<t!(=)>()?;
        let rhs = parser.parse_node::<Expr>()?;
        parser.consume::<t!(;)>()?;

        Ok(Self { var, ttype, rhs })
    }
}

impl Analyze for LetStmt {
    fn build(&self, builder: &mut crate::semantic_analyzer::scope_builder::ScopeBuilder) {
        let type_info = if self.ttype.inner == TypeInner::Unknown {
            builder.register_type(&TypeSpanned::dummy(Type {
                mutable: self.ttype.mutable,
                inner: TypeInner::Void,
            }))
        } else {
            builder.register_type(&self.ttype)
        };
        builder.define_var(&self.var, type_info)
    }

    fn analyze_semantics(&self, analyzer: &mut crate::semantic_analyzer::analyzer::Analyzer) {
        let Some(expr_type) = analyzer.resolve_expr(&self.rhs) else {
            return;
        };

        get_symbol_mut!((analyzer, self.var) info {
        if self.ttype.inner == TypeInner::Unknown {
            info.infer_type(&expr_type);
        }

        if !info.ty.assign_compatible(&expr_type.ttype) {
            analyzer.report_semantic_error(
                SemanticError::LetTypeMismatch(info.ty.ttype.clone(), expr_type.clone()),
                self.rhs.span(),
            );
        }

        });
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
