use crate::parser::{expr::Expr, parse_trait::Parse};
use crate::t;

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStmt {
    expr: Expr,
}

impl Parse for ReturnStmt {
    fn parse(token_stream: &mut crate::lexer::Lexer) -> crate::aliases::Result<Self> {
        token_stream.consume::<t!(return)>()?;
        let expr = token_stream.parse::<Expr>()?;
        token_stream.consume::<t!(;)>()?;
        Ok(Self { expr })
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;

    use super::ReturnStmt;

    #[test]
    fn test_return_stmt() {
        let mut lexer = Lexer::new("return a == b").unwrap();
        lexer.parse::<ReturnStmt>().unwrap();
    }
}
