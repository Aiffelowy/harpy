use crate::{
    lexer::tokens::Ident,
    parser::{expr::Expr, parse_trait::Parse, types::Type},
    t,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LetStmt {
    var: Ident,
    ttype: Type,
    rhs: Expr,
}

impl Parse for LetStmt {
    fn parse(token_stream: &mut crate::lexer::Lexer) -> crate::aliases::Result<Self> {
        token_stream.consume::<t!(let)>()?;
        let var = token_stream.consume::<t!(ident)>()?;
        token_stream.consume::<t!(:)>()?;
        let ttype = token_stream.parse::<Type>()?;
        token_stream.consume::<t!(=)>()?;
        let rhs = token_stream.parse::<Expr>()?;
        token_stream.consume::<t!(;)>()?;

        Ok(Self { var, ttype, rhs })
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;

    use super::LetStmt;

    #[test]
    fn test_let_stmt() {
        let mut lexer = Lexer::new("let var: mut int = (7 + 3) * 4;").unwrap();
        lexer.parse::<LetStmt>().unwrap();
    }
}
