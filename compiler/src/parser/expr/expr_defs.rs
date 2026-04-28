use crate::{
    lexer::tokens::{Ident, Literal},
    parser::{expr::ops::*, types::type_parsing::Type, Node},
};

pub struct CallExpr {
    pub callee: Box<Node<Expr>>,
    pub args: Vec<Node<Expr>>,
}

pub struct BlockExpr {
    pub exprs: Vec<Node<Expr>>,
}

pub struct LoopExpr {
    pub block: BlockExpr,
}

pub struct IfExpr {
    pub expr: Box<Node<Expr>>,
    pub block: BlockExpr,
    pub else_block: Option<Box<Node<Expr>>>,
}

pub struct ClosureExpr {
    pub args: Vec<(Ident, Node<Type>)>,
    pub return_type: Node<Type>,
    pub block: BlockExpr,
}

pub struct CaseExpr {
    pub expr: Node<Expr>,
    pub block: BlockExpr,
}

pub struct SwitchExpr {
    pub expr: Box<Node<Expr>>,
    pub cases: Vec<Node<CaseExpr>>,
}

pub struct BreakExpr {
    pub expr: Option<Node<Expr>>,
}

pub struct ReturnExpr {
    pub expr: Option<Node<Expr>>,
}

pub enum Expr {
    Infix(Box<Node<Expr>>, InfixOp, Box<Node<Expr>>),
    Prefix(PrefixOp, Box<Node<Expr>>),
    Assign(Box<Node<Expr>>, AssignOp, Box<Node<Expr>>),
    Literal(Literal),
    Ident(Ident),
    Call(Node<CallExpr>),
    Loop(Node<LoopExpr>),
    Return(Option<Box<Node<Expr>>>),
    Break(Option<Box<Node<Expr>>>),
    Continue,
    If(Node<IfExpr>),
    Switch(Node<SwitchExpr>),
    Block(Node<BlockExpr>),
    Closure(Node<ClosureExpr>),
    Borrow(Box<Node<Expr>>, bool),
    Box(Box<Node<Expr>>),
    Iter(Box<Node<Expr>>, Box<Node<Expr>>),
}
