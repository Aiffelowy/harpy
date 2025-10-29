use crate::lexer::err::LexerError;
use crate::lexer::tokens::Ident;
use crate::t;
use crate::tt;
use crate::{
    aliases::Result,
    lexer::{tokens::Literal, Lexer},
    parser::Parse,
};

use super::infix::InfixOp;
use super::prefix::PrefixOp;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Infix(Box<Expr>, InfixOp, Box<Expr>),
    Prefix(PrefixOp, Box<Expr>),
    Literal(Literal),
    Ident(Ident),
    Call(Ident, Vec<Expr>),
}

impl Expr {
    fn parse_expr(token_stream: &mut Lexer, min_bp: u8) -> Result<Self> {
        let mut lhs = Expr::parse_null_den(token_stream)?;

        loop {
            let mut fork = token_stream.fork();
            let Ok(op) = fork.parse::<InfixOp>() else {
                break;
            };

            let bp = op.bp();
            if bp.left < min_bp {
                break;
            }

            token_stream.parse::<InfixOp>()?;

            let rhs = Expr::parse_expr(token_stream, bp.right)?;
            lhs = Expr::Infix(Box::new(lhs), op, Box::new(rhs));
        }

        Ok(lhs)
    }

    fn parse_null_den(token_stream: &mut Lexer) -> Result<Self> {
        match token_stream.peek()? {
            tt!(lit) => {
                let val = token_stream.consume::<t!(lit)>()?;
                return Ok(Expr::Literal(val));
            }
            tt!(ident) => {
                let ident = token_stream.consume::<t!(ident)>()?;
                if *token_stream.peek()? != tt!("(") {
                    return Ok(Expr::Ident(ident));
                }

                token_stream.consume::<t!("(")>()?;
                let mut args = vec![];
                loop {
                    if *token_stream.peek()? == tt!(")") {
                        break;
                    }

                    args.push(Expr::parse_expr(token_stream, 0)?);
                    if *token_stream.peek()? == tt!(,) {
                        token_stream.consume::<t!(,)>()?;
                    } else {
                        break;
                    }
                }
                token_stream.consume::<t!(")")>()?;
                return Ok(Expr::Call(ident, args));
            }
            tt!("(") => {
                token_stream.consume::<t!("(")>()?;
                let expr = Expr::parse_expr(token_stream, 0)?;
                token_stream.consume::<t!(")")>()?;
                return Ok(expr);
            }
            _ => (),
        }

        if let Some(op) = token_stream.try_parse::<PrefixOp>() {
            let rhs = Expr::parse_expr(token_stream, op.bp().right)?;
            return Ok(Expr::Prefix(op, Box::new(rhs)));
        }

        return Err(LexerError::UnexpectedToken("expression", token_stream.next_token()?).into());
    }
}

impl Parse for Expr {
    fn parse(token_stream: &mut crate::lexer::Lexer) -> crate::aliases::Result<Self> {
        Expr::parse_expr(token_stream, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse_expr(s: &str) -> Expr {
        let mut lexer = Lexer::new(s).unwrap();
        lexer.parse::<Expr>().unwrap()
    }

    #[test]
    fn test_literal() {
        let expr = parse_expr("42");
        assert!(matches!(expr, Expr::Literal(_)));
    }

    #[test]
    fn test_ident() {
        let expr = parse_expr("foo");
        assert!(matches!(expr, Expr::Ident(_)));
    }

    #[test]
    fn test_prefix_box() {
        let expr = parse_expr("box 42");
        match expr {
            Expr::Prefix(PrefixOp::Box, inner) => {
                assert!(matches!(*inner, Expr::Literal(_)));
            }
            _ => panic!("Expected box prefix expression"),
        }
    }

    #[test]
    fn test_prefix_minus() {
        let expr = parse_expr("-5");
        match expr {
            Expr::Prefix(PrefixOp::Minus, inner) => {
                assert!(matches!(*inner, Expr::Literal(_)));
            }
            _ => panic!("Expected minus prefix expression"),
        }
    }

    #[test]
    fn test_infix_simple() {
        let expr = parse_expr("1 + 2");
        match expr {
            Expr::Infix(lhs, InfixOp::Plus, rhs) => {
                assert!(matches!(*lhs, Expr::Literal(_)));
                assert!(matches!(*rhs, Expr::Literal(_)));
            }
            _ => panic!("Expected 1 + 2 infix"),
        }
    }

    #[test]
    fn test_infix_precedence() {
        let expr = parse_expr("1 + 2 * 3");
        match expr {
            Expr::Infix(lhs, InfixOp::Plus, rhs) => {
                assert!(matches!(*lhs, Expr::Literal(_)));
                match *rhs {
                    Expr::Infix(_, InfixOp::Mult, _) => {} // correct precedence
                    _ => panic!("Expected 2 * 3 on RHS"),
                }
            }
            _ => panic!("Expected 1 + (2 * 3)"),
        }
    }

    #[test]
    fn test_parentheses() {
        let expr = parse_expr("(1 + 2) * 3");
        match expr {
            Expr::Infix(lhs, InfixOp::Mult, rhs) => {
                match *lhs {
                    Expr::Infix(_, InfixOp::Plus, _) => {} // parentheses worked
                    _ => panic!("Expected (1+2) on LHS"),
                }
                assert!(matches!(*rhs, Expr::Literal(_)));
            }
            _ => panic!("Expected (1+2)*3"),
        }
    }

    #[test]
    fn test_nested_box() {
        let expr = parse_expr("box (3 + 4) * 2");
        match expr {
            Expr::Prefix(PrefixOp::Box, inner) => match *inner {
                Expr::Infix(lhs, InfixOp::Mult, rhs) => {
                    match *lhs {
                        Expr::Infix(_, InfixOp::Plus, _) => {}
                        _ => panic!("Expected (3 + 4) on LHS"),
                    }
                    assert!(matches!(*rhs, Expr::Literal(_)));
                }
                _ => panic!("Expected (3 + 4) * 2 inside box"),
            },
            _ => panic!("Expected box ((3 + 4) * 2), got {:?}", expr),
        }
    }

    #[test]
    fn test_call_expr() {
        let expr = parse_expr("foo(1, 2 + 3)");
        match expr {
            Expr::Call(ident, args) => {
                assert_eq!(ident.value(), "foo");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected function call"),
        }
    }

    #[test]
    fn test_complex_expr() {
        let expr = parse_expr("box ((3.5 + 2.3) * 3 + 1) / 3.1");
        // just ensure it parses without panicking
        println!("{:#?}", expr);
    }
}
