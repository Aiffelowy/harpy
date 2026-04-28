use crate::lexer::span::Span;
use crate::lexer::tokens::Ident;
use crate::parser::expr::expr_defs::*;
use crate::parser::expr::ops::{AssignOp, InfixOp, PrefixOp};
use crate::parser::types::type_parsing::Type;
use crate::parser::{Node, Parser};
use crate::{aliases::Result, lexer::tokens::TokenType, t, tt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::parser) enum Precedence {
    Lowest,
    Assign,
    Range,
    Or,
    And,
    Compare,
    Sum,
    Mul,
    Prefix,
    Call,
}

impl Precedence {
    fn get(token: &TokenType) -> Self {
        match token {
            tt!(=) | tt!(+=) | tt!(-=) | tt!(*=) | tt!(/=) | tt!(%=) => Self::Assign,
            tt!(=>) => Self::Range,
            tt!(||) => Self::Or,
            tt!(&&) => Self::And,
            tt!(==) | tt!(>) | tt!(<) | tt!(<=) | tt!(>=) | tt!(!=) => Self::Compare,
            tt!(+) | tt!(-) => Self::Sum,
            tt!(*) | tt!(/) | tt!(%) => Self::Mul,
            tt!("(") | tt!(.) => Self::Call,
            _ => Self::Lowest,
        }
    }
}

impl<'parser> Parser<'parser> {
    fn parse_call_expr(&mut self, callee: Node<Expr>) -> Result<CallExpr> {
        let mut args = Vec::new();
        self.consume::<t!("(")>()?;
        loop {
            if let tt!(")") | tt!(eof) = self.peek()? {
                break;
            }

            args.push(self.parse_node(Self::parse_expr)?);
            if let tt!(,) = self.peek()? {
                self.consume::<t!(,)>()?;
            } else {
                break;
            }
        }

        self.consume::<t!(")")>()?;

        Ok(CallExpr {
            callee: Box::new(callee),
            args,
        })
    }

    pub(in crate::parser) fn parse_block_expr(&mut self) -> Result<BlockExpr> {
        self.consume::<t!("{")>()?;
        let mut stmts = Vec::new();

        loop {
            if let tt!("}") = self.peek()? {
                break;
            }

            match self.parse_node(Self::parse_stmt) {
                Ok(stmt) => stmts.push(stmt),
                Err(e) => self.report_error(e, &[])?,
            }
        }

        self.consume::<t!("}")>()?;
        Ok(BlockExpr { stmts })
    }

    fn parse_loop_expr(&mut self) -> Result<LoopExpr> {
        self.consume::<t!(loop)>()?;
        let block = self.parse_block_expr()?;
        Ok(LoopExpr { block })
    }

    fn parse_if_expr(&mut self) -> Result<IfExpr> {
        self.consume::<t!(if)>()?;

        let expr = Box::new(self.parse_node(Self::parse_expr)?);
        let block = self.parse_node(Self::parse_block_expr)?;
        let mut else_block = None;
        if let tt!(else) = self.peek()? {
            self.consume::<t!(else)>()?;
            else_block = Some(Box::new(self.parse_node(Self::parse_expr)?));
        }

        Ok(IfExpr {
            expr,
            block,
            else_block,
        })
    }

    pub(in crate::parser) fn parse_function_arg(&mut self) -> Result<(Ident, Node<Type>)> {
        let name = self.consume()?;
        self.consume::<t!(:)>()?;
        let ttype = self.parse_node(Self::parse_type)?;

        Ok((name, ttype))
    }

    pub(in crate::parser) fn parse_function_return_type(&mut self) -> Result<Type> {
        if self.peek()? != &tt!(->) {
            return Ok(Type::void());
        }

        self.consume::<t!(->)>()?;
        self.parse_type()
    }

    fn parse_closure_expr(&mut self) -> Result<ClosureExpr> {
        self.consume::<t!(fn)>()?;
        self.consume::<t!("(")>()?;
        let mut args = vec![];
        loop {
            if let tt!(")") | tt!(eof) = self.peek()? {
                break;
            }

            args.push(self.parse_function_arg()?);
            if let tt!(,) = self.peek()? {
                self.consume::<t!(,)>()?;
            } else {
                break;
            }
        }

        self.consume::<t!(")")>()?;

        let return_type = self.parse_node(Self::parse_function_return_type)?;

        let block = self.parse_node(Self::parse_block_expr)?;

        Ok(ClosureExpr {
            args,
            return_type,
            block,
        })
    }

    fn parse_case_expr(&mut self) -> Result<CaseExpr> {
        let expr = self.parse_node(Self::parse_expr)?;
        self.consume::<t!(=>)>()?;
        let block = self.parse_node(Self::parse_block_expr)?;

        Ok(CaseExpr { expr, block })
    }

    fn parse_switch_expr(&mut self) -> Result<SwitchExpr> {
        self.consume::<t!(switch)>()?;
        let expr = Box::new(self.parse_node(Self::parse_expr)?);
        let mut cases = vec![];
        self.consume::<t!("{")>()?;

        loop {
            if let tt!("}") = self.peek()? {
                break;
            }

            match self.parse_node(Self::parse_case_expr) {
                Ok(case) => cases.push(case),
                Err(e) => self.report_error(e, &[])?,
            }
        }

        self.consume::<t!("}")>()?;

        Ok(SwitchExpr { expr, cases })
    }

    fn parse_prefix(&mut self) -> Result<Expr> {
        match self.peek()? {
            tt!(lit) => {
                let lit = self.consume()?;
                Ok(Expr::Literal(lit))
            }
            tt!(ident) => {
                let name = self.consume()?;

                if let tt!("{") = self.peek()? {
                    self.consume::<t!("{")>()?;
                    let mut fields = Vec::new();
                    loop {
                        if let tt!("}") | tt!(eof) = self.peek()? {
                            break;
                        }

                        let field_name = self.consume()?;
                        self.consume::<t!(:)>()?;
                        let field_value = self.parse_node(Self::parse_expr)?;
                        fields.push((field_name, field_value));

                        if let tt!(,) = self.peek()? {
                            self.consume::<t!(,)>()?;
                        } else {
                            break;
                        }
                    }
                    self.consume::<t!("}")>()?;
                    Ok(Expr::StructInit(name, fields))
                } else {
                    Ok(Expr::Ident(name))
                }
            }

            tt!(-) | tt!(+) | tt!(!) | tt!(*) => {
                let op = match self.peek()? {
                    tt!(-) => {
                        self.consume::<t!(-)>()?;
                        PrefixOp::Minus
                    }
                    tt!(+) => {
                        self.consume::<t!(+)>()?;
                        PrefixOp::Plus
                    }
                    tt!(!) => {
                        self.consume::<t!(!)>()?;
                        PrefixOp::Neg
                    }
                    tt!(*) => {
                        self.consume::<t!(*)>()?;
                        PrefixOp::Star
                    }
                    _ => unreachable!(),
                };

                let right = self.parse_node(|p| p.pratt_parser(Precedence::Prefix))?;
                Ok(Expr::Prefix(op, Box::new(right)))
            }

            tt!("(") => {
                self.consume::<t!("(")>()?;
                let inner = self.parse_expr()?;
                self.consume::<t!(")")>()?;
                Ok(inner)
            }
            tt!("{") => Ok(Expr::Block(self.parse_node(Self::parse_block_expr)?)),
            tt!(if) => Ok(Expr::If(self.parse_node(Self::parse_if_expr)?)),
            tt!(loop) => Ok(Expr::Loop(self.parse_node(Self::parse_loop_expr)?)),
            tt!(switch) => Ok(Expr::Switch(self.parse_node(Self::parse_switch_expr)?)),
            tt!(fn) => Ok(Expr::Closure(self.parse_node(Self::parse_closure_expr)?)),
            tt!(return) => {
                self.consume::<t!(return)>()?;
                let mut value = None;
                if self.peek()? != &tt!(;) {
                    value = Some(Box::new(self.parse_node(Self::parse_expr)?))
                }
                Ok(Expr::Return(value))
            }
            tt!(break) => {
                self.consume::<t!(break)>()?;
                let mut value = None;
                if self.peek()? != &tt!(;) {
                    value = Some(Box::new(self.parse_node(Self::parse_expr)?))
                }
                Ok(Expr::Break(value))
            }
            tt!(continue) => {
                self.consume::<t!(continue)>()?;
                Ok(Expr::Continue)
            }

            tt!(ref) => {
                self.consume::<t!(ref)>()?;
                let mut is_mut = false;
                if let tt!(mut) = self.peek()? {
                    self.consume::<t!(mut)>()?;
                    is_mut = true;
                }
                let right = self.parse_node(|p| p.pratt_parser(Precedence::Prefix))?;
                Ok(Expr::Borrow(Box::new(right), is_mut))
            }

            tt!(box) => {
                self.consume::<t!(box)>()?;
                let right = self.parse_node(|p| p.pratt_parser(Precedence::Prefix))?;
                Ok(Expr::Box(Box::new(right)))
            }

            _ => self.unexpected("prefix token in expression"),
        }
    }

    fn parse_infix_op(&mut self) -> Result<InfixOp> {
        let op = match self.peek()? {
            tt!(+) => InfixOp::Add,
            tt!(-) => InfixOp::Sub,
            tt!(*) => InfixOp::Mul,
            tt!(/) => InfixOp::Div,
            tt!(%) => InfixOp::Mod,
            tt!(&&) => InfixOp::And,
            tt!(||) => InfixOp::Or,
            tt!(>) => InfixOp::Gt,
            tt!(<) => InfixOp::Lt,
            tt!(==) => InfixOp::Eq,
            tt!(>=) => InfixOp::GtEq,
            tt!(<=) => InfixOp::LtEq,
            tt!(!=) => InfixOp::Neq,
            _ => {
                return self.unexpected("infix operator");
            }
        };

        self.discard_next()?;
        Ok(op)
    }

    fn parse_assign_op(&mut self) -> Result<AssignOp> {
        let op = match self.peek()? {
            tt!(=) => AssignOp::Eq,
            tt!(+=) => AssignOp::Add,
            tt!(-=) => AssignOp::Sub,
            tt!(*=) => AssignOp::Mul,
            tt!(/=) => AssignOp::Div,
            tt!(%=) => AssignOp::Mod,
            _ => {
                return self.unexpected("assign operator");
            }
        };

        self.discard_next()?;
        Ok(op)
    }

    fn parse_infix(&mut self, left: Node<Expr>) -> Result<Node<Expr>> {
        let start = left.span.start;
        let op_tok = self.peek()?.clone();
        let prec = Precedence::get(&op_tok);

        let expr = match op_tok {
            tt!(+)
            | tt!(-)
            | tt!(*)
            | tt!(/)
            | tt!(%)
            | tt!(||)
            | tt!(&&)
            | tt!(>)
            | tt!(<)
            | tt!(==)
            | tt!(>=)
            | tt!(<=)
            | tt!(!=) => {
                let op = self.parse_infix_op()?;
                let right = self.parse_node(|p| p.pratt_parser(prec))?;
                Expr::Infix(Box::new(left), op, Box::new(right))
            }

            tt!(=>) => {
                self.consume::<t!(=>)>()?;
                let right = self.parse_node(|p| p.pratt_parser(prec))?;
                Expr::Iter(Box::new(left), Box::new(right))
            }

            tt!(.) => {
                self.consume::<t!(.)>()?;
                let name = self.consume()?;
                Expr::MemberAccess(Box::new(left), name)
            }

            tt!(=) | tt!(+=) | tt!(-=) | tt!(*=) | tt!(/=) | tt!(%=) => {
                let op = self.parse_assign_op()?;
                let right = self.parse_node(|p| p.pratt_parser(Precedence::Lowest))?;
                Expr::Assign(Box::new(left), op, Box::new(right))
            }

            tt!("(") => {
                let call = self.parse_node(|p| p.parse_call_expr(left))?;
                Expr::Call(call)
            }

            _ => unreachable!("precedence should prevent this!"),
        };

        let end = self.previous_end;
        Ok(Node {
            id: self.next_id(),
            span: Span::new(start, end),
            inner: expr,
        })
    }

    fn pratt_parser(&mut self, precedence: Precedence) -> Result<Expr> {
        let mut left = self.parse_node(Self::parse_prefix)?;
        while precedence < Precedence::get(self.peek()?) {
            left = self.parse_infix(left)?;
        }

        Ok(left.inner)
    }

    pub(in crate::parser) fn parse_expr(&mut self) -> Result<Expr> {
        self.pratt_parser(Precedence::Lowest)
    }
}
