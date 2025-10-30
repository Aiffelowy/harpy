use crate::parser::parser::Parser;
use crate::parser::{expr::Expr, parse_trait::Parse};
use crate::{t, tt};

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStmt {
    expr: Option<Expr>,
}

impl Parse for ReturnStmt {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        parser.consume::<t!(return)>()?;

        if let tt!(;) = parser.peek()? {
            parser.consume::<t!(;)>()?;
            return Ok(Self { expr: None });
        }

        let expr = parser.parse::<Expr>()?;
        parser.consume::<t!(;)>()?;
        Ok(Self { expr: Some(expr) })
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
