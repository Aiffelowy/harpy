use crate::{
    lexer::tokens::Ident,
    parser::{expr::Expr, parse_trait::Parse, parser::Parser, types::Type},
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
