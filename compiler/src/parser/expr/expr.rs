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
    Box(Box<Node<Expr>>),
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
            tt!(box) => {
                parser.consume::<t!(box)>()?;
                let expr = parser.parse_node()?;
                return Ok(Expr::Box(Box::new(expr)));
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
            Expr::Box(expr) => expr.span(),
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
            Expr::Box(_) => None,
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
            Expr::Box(expr) => format!("box {expr}"),
        };

        write!(f, "{s}")
    }
}
