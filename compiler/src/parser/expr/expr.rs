use crate::err::{HarpyError, Kind};
use crate::lexer::span::Span;
use crate::parser::expr::expr_defs::*;
use crate::parser::expr::ops::{AssignOp, InfixOp, PrefixOp};
use crate::parser::stmt::stmts::RichStmt;
use crate::parser::types::type_parsing::Type;
use crate::parser::{Node, Parser};
use crate::{aliases::Result, lexer::tokens::TokenType, t, tt};
use crate::{parse_separated, parse_sequence, peek_and_consume};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::parser) enum Precedence {
    Lowest,
    Assign,
    //Range,
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
            //tt!(=>) => Self::Range,
            tt!(||) => Self::Or,
            tt!(&&) => Self::And,
            tt!(==) | tt!(>) | tt!(<) | tt!(<=) | tt!(>=) | tt!(!=) => Self::Compare,
            tt!(+) | tt!(-) => Self::Sum,
            tt!(*) | tt!(/) | tt!(%) => Self::Mul,
            tt!("(") | tt!(.) | tt!("[") => Self::Call,
            _ => Self::Lowest,
        }
    }
}

impl<'parser> Parser<'parser> {
    fn parse_call_expr(&mut self, callee: Node<RichExpr>) -> Result<RichCallExpr> {
        let args = parse_separated!(self, "(", ")", ,, self.parse_node(Self::parse_expr)?);
        Ok(RichCallExpr { callee, args })
    }

    pub(in crate::parser) fn parse_block_expr(&mut self) -> Result<RichBlockExpr> {
        let mut stmts = parse_sequence!(self, "{", "}", self.parse_node(Self::parse_stmt), []);
        let mut tail = None;

        if let Some(last) = stmts.pop() {
            let id = last.id();
            let (stmt, span) = last.extract();
            match stmt {
                RichStmt::Expr(expr) => tail = Some(expr),
                other => stmts.push(Node::new(id, span, other)),
            }
        }

        Ok(RichBlockExpr { stmts, tail })
    }

    fn parse_loop_expr(&mut self) -> Result<RichLoopExpr> {
        self.consume::<t!(loop)>()?;
        let block = self.parse_node(Self::parse_block_expr)?;
        Ok(RichLoopExpr { block })
    }

    fn parse_if_expr(&mut self) -> Result<RichIfExpr> {
        self.consume::<t!(if)>()?;

        let expr = self.parse_node(Self::parse_expr)?;
        let block = self.parse_node(Self::parse_block_expr)?;
        let mut else_block = None;
        if peek_and_consume!(self, else) {
            else_block = Some(self.parse_node(Self::parse_expr)?);
        }

        Ok(RichIfExpr {
            expr,
            block,
            else_block,
        })
    }

    pub(in crate::parser) fn parse_function_arg(&mut self) -> Result<FunctionArg> {
        let mutable = peek_and_consume!(self, mut);
        let name = self.consume()?;
        self.consume::<t!(:)>()?;
        let ty = self.parse_node(Self::parse_type)?;

        Ok(FunctionArg { name, ty, mutable })
    }

    pub(in crate::parser) fn parse_function_return_type(&mut self) -> Result<Type> {
        if self.peek()? != &tt!(->) {
            return Ok(Type::void());
        }

        self.consume::<t!(->)>()?;
        self.parse_type()
    }

    fn parse_closure_expr(&mut self) -> Result<RichClosureExpr> {
        self.consume::<t!(fn)>()?;
        let args = parse_separated!(self, "(", ")",,, self.parse_function_arg()?);
        let return_type = self.parse_node(Self::parse_function_return_type)?;

        let block = self.parse_node(Self::parse_block_expr)?;

        Ok(RichClosureExpr {
            args,
            return_type,
            block,
        })
    }

    fn parse_switch_expr(&mut self) -> Result<SwitchExpr> {
        self.consume::<t!(switch)>()?;
        let expr = self.parse_node(Self::parse_expr)?;
        self.consume::<t!("{")>()?;

        let mut cases = Vec::new();
        let mut default = None;

        loop {
            let peeked = self.peek()?;
            if let tt!("}") = peeked {
                self.consume::<t!("}")>()?;
                break;
            }

            if let tt!(eof) = peeked {
                break;
            }

            if let tt!(.) = self.peek()? {
                let dot = self.consume::<t!(.)>()?;
                self.consume::<t!(=>)>()?;
                let block = self.parse_node(Self::parse_block_expr)?;
                if default.is_some() {
                    return HarpyError::err(dot.span(), Kind::MultipleDefaultInSwitch);
                }
                default = Some(block)
            } else {
                let expr = self.parse_node(Self::parse_expr)?;
                self.consume::<t!(=>)>()?;
                let block = self.parse_node(Self::parse_block_expr)?;
                cases.push((expr, block));
            }
        }

        Ok(SwitchExpr {
            expr,
            cases,
            default,
        })
    }

    fn parse_prefix(&mut self) -> Result<RichExpr> {
        match self.peek()? {
            tt!(lit) => {
                let lit = self.consume()?;
                Ok(RichExpr::Literal(lit))
            }
            tt!(ident) => {
                let name = self.consume()?;

                Ok(RichExpr::Ident(name))
            }
            tt!(.) => {
                self.consume::<t!(.)>()?;
                Ok(RichExpr::Implicit)
            }

            // hacky special syntax for spawn caged Object (sounds really cool ok?)
            tt!(spawn) => {
                let spawn = self.consume::<t!(spawn)>()?;

                let is_boxed = peek_and_consume!(self, boxed);
                let is_mut = is_boxed && peek_and_consume!(self, mut);
                let name = self.consume()?;
                let fields = parse_separated!(self, "{", "}",;, {
                    let field_name = self.consume()?;
                    self.consume::<t!(=)>()?;
                    let field_value = self.parse_node(Self::parse_expr)?;
                    (field_name, field_value)
                });

                if is_boxed {
                    let span = Span::new(spawn.span().start, self.previous_end, self.file_id());
                    let node = Node::new(self.next_id(), span, RichExpr::StructInit(name, fields));
                    Ok(RichExpr::Box(node, is_mut))
                } else {
                    Ok(RichExpr::StructInit(name, fields))
                }
            }

            tt!(-) | tt!(+) | tt!(!) => {
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
                    _ => unreachable!(),
                };

                let right = self.parse_node(|p| p.pratt_parser(Precedence::Prefix))?;
                Ok(RichExpr::Prefix(op, right))
            }

            tt!("(") => {
                self.consume::<t!("(")>()?;
                let inner = self.parse_expr()?;
                self.consume::<t!(")")>()?;
                Ok(inner)
            }
            tt!("[") => {
                let elements =
                    parse_separated!(self, "[", "]",,, self.parse_node(Self::parse_expr)?);
                Ok(RichExpr::ArrayInit(elements))
            }
            tt!("{") => Ok(RichExpr::Block(self.parse_node(Self::parse_block_expr)?)),
            tt!(if) => Ok(RichExpr::If(self.parse_node(Self::parse_if_expr)?)),
            tt!(loop) => Ok(RichExpr::Loop(self.parse_node(Self::parse_loop_expr)?)),
            tt!(switch) => Ok(RichExpr::Switch(self.parse_node(Self::parse_switch_expr)?)),
            tt!(fn) => Ok(RichExpr::Closure(
                self.parse_node(Self::parse_closure_expr)?,
            )),
            tt!(return) => {
                self.consume::<t!(return)>()?;
                let mut value = None;
                if self.peek()? != &tt!(;) {
                    value = Some(self.parse_node(Self::parse_expr)?)
                }
                Ok(RichExpr::Return(value))
            }
            tt!(break) => {
                self.consume::<t!(break)>()?;
                let mut value = None;
                if self.peek()? != &tt!(;) {
                    value = Some(self.parse_node(Self::parse_expr)?)
                }
                Ok(RichExpr::Break(value))
            }
            tt!(continue) => {
                self.consume::<t!(continue)>()?;
                Ok(RichExpr::Continue)
            }

            tt!(ref) => {
                self.consume::<t!(ref)>()?;
                let is_mut = peek_and_consume!(self, mut);
                let right = self.parse_node(|p| p.pratt_parser(Precedence::Prefix))?;
                Ok(RichExpr::Borrow(right, is_mut))
            }

            tt!(box) => {
                self.consume::<t!(box)>()?;
                let is_mut = peek_and_consume!(self, mut);
                let right = self.parse_node(|p| p.pratt_parser(Precedence::Prefix))?;
                Ok(RichExpr::Box(right, is_mut))
            }
            tt!(clone) => {
                self.consume::<t!(clone)>()?;
                let expr = self.parse_node(Self::parse_expr)?;
                Ok(RichExpr::Clone(expr))
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

    fn parse_infix(&mut self, left: Node<RichExpr>) -> Result<Node<RichExpr>> {
        let start = left.span().start;
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
                RichExpr::Infix(left, op, right)
            }

            //tt!(=>) => {
            //    self.consume::<t!(=>)>()?;
            //    let right = self.parse_node(|p| p.pratt_parser(prec))?;

            //    let start = match left.inner() {
            //        RichExpr::Implicit => None,
            //        _ => Some(left),
            //    };

            //    let end = match right.inner() {
            //        RichExpr::Implicit => None,
            //        _ => Some(right),
            //    };
            //    RichExpr::Range(start, end)
            //}
            tt!(.) => {
                self.consume::<t!(.)>()?;
                let name = self.consume()?;
                RichExpr::MemberAccess(left, name)
            }

            tt!("[") => {
                self.consume::<t!("[")>()?;
                let index = self.parse_node(Self::parse_expr)?;
                self.consume::<t!("]")>()?;
                RichExpr::Index(left, index)
            }

            tt!(=) | tt!(+=) | tt!(-=) | tt!(*=) | tt!(/=) | tt!(%=) => {
                let op = self.parse_assign_op()?;
                let right = self.parse_node(Self::parse_expr)?;
                RichExpr::Assign(left, op, right)
            }

            tt!("(") => {
                let call = self.parse_node(|p| p.parse_call_expr(left))?;
                RichExpr::Call(call)
            }

            _ => unreachable!("precedence should prevent this!"),
        };

        let end = self.previous_end;
        Ok(Node::new(
            self.next_id(),
            Span::new(start, end, self.file_id()),
            expr,
        ))
    }

    fn pratt_parser(&mut self, precedence: Precedence) -> Result<RichExpr> {
        let mut left = self.parse_node(Self::parse_prefix)?;
        while precedence < Precedence::get(self.peek()?) {
            left = self.parse_infix(left)?;
        }
        let (left, _) = left.extract();
        Ok(left)
    }

    pub(in crate::parser) fn parse_expr(&mut self) -> Result<RichExpr> {
        self.pratt_parser(Precedence::Lowest)
    }
}
