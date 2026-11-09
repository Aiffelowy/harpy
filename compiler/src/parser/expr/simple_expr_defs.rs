use crate::{
    lexer::tokens::{Ident, Literal},
    parser::{
        expr::{expr_defs::FunctionArg, ops::*},
        stmt::stmts::Stmt,
        types::type_parsing::Type,
        Node,
    },
};

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub callee: Node<Expr>,
    pub args: Vec<Node<Expr>>,
}

#[derive(Debug, Clone)]
pub struct BlockExpr {
    pub stmts: Vec<Node<Stmt>>,
    pub tail: Option<Node<Expr>>,
}

#[derive(Debug, Clone)]
pub struct LoopExpr {
    pub block: Node<BlockExpr>,
}

#[derive(Debug, Clone)]
pub struct IfExpr {
    pub expr: Node<Expr>,
    pub block: Node<BlockExpr>,
    pub else_block: Option<Node<Expr>>,
}

#[derive(Debug, Clone)]
pub struct ClosureExpr {
    pub args: Vec<FunctionArg>,
    pub return_type: Node<Type>,
    pub block: Node<BlockExpr>,
}

#[derive(Debug, Clone)]
pub struct BreakExpr {
    pub expr: Option<Node<Expr>>,
}

#[derive(Debug, Clone)]
pub struct ReturnExpr {
    pub expr: Option<Node<Expr>>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Infix(Node<Expr>, InfixOp, Node<Expr>),
    Prefix(PrefixOp, Node<Expr>),
    Assign(Node<Expr>, Node<Expr>),
    Literal(Literal),
    Ident(Ident),
    Call(Node<CallExpr>),
    Loop(Node<LoopExpr>),
    Return(Option<Node<Expr>>),
    Break(Option<Node<Expr>>),
    Continue,
    If(Node<IfExpr>),
    Block(Node<BlockExpr>),
    Closure(Node<ClosureExpr>),
    Borrow(Node<Expr>, bool),
    Box(Node<Expr>, bool),
    Implicit,
    Range(Option<Node<Expr>>, Option<Node<Expr>>),
    MemberAccess(Node<Expr>, Ident),
    StructInit(Ident, Vec<(Ident, Node<Expr>)>),
    ArrayInit(Vec<Node<Expr>>),
    Index(Node<Expr>, Node<Expr>),
    Clone(Node<Expr>),
}
