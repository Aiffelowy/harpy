use std::fmt::Display;
use std::ops::Deref;

use crate::lexer::span::Span;
use crate::lexer::tokens::Ident;
use crate::parser::node::Node;
use crate::parser::parser::Parser;
use crate::t;
use crate::tt;
use crate::{aliases::Result, lexer::tokens::Literal, parser::Parse};

use super::infix::InfixOp;
use super::prefix::PrefixOp;
use super::prefix::PrefixOpKind;

#[derive(Debug, Clone)]
pub struct SpannedExpr {
    expr: Expr,
    span: Span,
}

impl Parse for SpannedExpr {
    fn parse(parser: &mut Parser) -> Result<Self> {
        let (expr, span) = parser.parse_spanned()?;
        Ok(Self { expr, span })
    }
}

impl Deref for SpannedExpr {
    type Target = Expr;
    fn deref(&self) -> &Self::Target {
        &self.expr
    }
}

impl Display for SpannedExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)
    }
}

impl SpannedExpr {
    pub fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub ident: Ident,
    pub args: Vec<Node<Expr>>,
}

impl Parse for CallExpr {
    fn parse(parser: &mut Parser) -> Result<Self> {
        let ident = parser.consume()?;
        parser.consume::<t!("(")>()?;
        let mut args = vec![];
        loop {
            if *parser.peek()? == tt!(")") {
                break;
            }

            args.push(parser.parse_node::<Expr>()?);
            if *parser.peek()? == tt!(,) {
                parser.consume::<t!(,)>()?;
            } else {
                break;
            }
        }
        parser.consume::<t!(")")>()?;

        Ok(Self { ident, args })
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Infix(Box<Expr>, InfixOp, Box<Expr>),
    Prefix(PrefixOp, Box<Expr>),
    Literal(Node<Literal>),
    Ident(Node<Ident>),
    Call(Node<CallExpr>),
    Borrow(Box<SpannedExpr>, bool),
}

impl Expr {
    fn parse_expr(parser: &mut Parser, min_bp: u8) -> Result<Self> {
        let mut lhs = Expr::parse_null_den(parser)?;

        loop {
            let mut fork = parser.fork();
            let Ok(op) = fork.parse::<InfixOp>() else {
                break;
            };

            let bp = op.bp();
            if bp.left < min_bp {
                break;
            }

            parser.parse::<InfixOp>()?;

            let rhs = Expr::parse_expr(parser, bp.right)?;
            lhs = Expr::Infix(Box::new(lhs), op, Box::new(rhs));
        }

        Ok(lhs)
    }

    fn parse_null_den(parser: &mut Parser) -> Result<Self> {
        match parser.peek()? {
            tt!(lit) => {
                let val = parser.parse_node()?;
                return Ok(Expr::Literal(val));
            }
            tt!(ident) => {
                let mut fork = parser.fork();
                fork.consume::<t!(ident)>()?;

                if let tt!("(") = *fork.peek()? {
                    let call = parser.parse_node::<CallExpr>()?;
                    return Ok(Expr::Call(call));
                }

                let ident = parser.parse_node::<t!(ident)>()?;
                return Ok(Expr::Ident(ident));
            }
            tt!(&) => {
                parser.consume::<t!(&)>()?;
                let mut mutable = false;
                if let tt!(mut) = parser.peek()? {
                    mutable = true;
                    parser.consume::<t!(mut)>()?;
                }
                let expr = parser.parse()?;
                return Ok(Expr::Borrow(Box::new(expr), mutable));
            }
            _ => (),
        }

        if let Some(op) = parser.try_parse::<PrefixOp>() {
            let rhs = Expr::parse_expr(parser, op.bp().right)?;
            return Ok(Expr::Prefix(op, Box::new(rhs)));
        }

        return parser.unexpected("expression");
    }
}

impl Parse for Expr {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        Expr::parse_expr(parser, 0)
    }
}

impl Expr {
    pub fn calc_span(&self) -> Span {
        match self {
            Expr::Ident(i) => i.span(),
            Expr::Prefix(op, expr) => Span::new(op.span().start, expr.calc_span().end),
            Expr::Infix(lhs, _, rhs) => Span::new(lhs.calc_span().start, rhs.calc_span().end),
            Expr::Literal(l) => l.span(),
            Expr::Borrow(expr, _) => expr.calc_span(),
            Expr::Call(expr) => expr.span(),
        }
    }

    pub fn lvalue(&self) -> Option<&Node<Ident>> {
        match self {
            Expr::Ident(i) => Some(i),
            Expr::Prefix(PrefixOp { op, .. }, expr) if *op == PrefixOpKind::Star => expr.lvalue(),
            Expr::Borrow(expr, _) => expr.lvalue(),

            Expr::Literal(_) => None,
            Expr::Call(_) => None,
            Expr::Infix(_, _, _) => None,
            Expr::Prefix(_, _) => None,
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Expr::Literal(l) => format!("{}", l.value()),
            Expr::Ident(i) => format!("{}", i.value()),
            Expr::Call(expr) => {
                let mut s = format!("{}(", expr.ident.value());
                for param in &expr.args {
                    s.push_str(&format!("{},", param));
                }
                s.push(')');
                s
            }
            Expr::Prefix(op, expr) => format!("{op}{expr}"),
            Expr::Infix(lhs, op, rhs) => format!("{lhs} {op} {rhs}"),
            Expr::Borrow(rhs, mutable) => format!("&{}{rhs}", if *mutable { "mut " } else { "" }),
        };

        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse_expr(s: &str) -> Expr {
        let mut parser = Parser::new(Lexer::new(s).unwrap());
        parser.parse::<Expr>().unwrap()
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
