use crate::{
    lexer::tokens::{Ident, Literal},
    parser::{
        expr::ops::*,
        stmt::stmts::Stmt,
        types::type_parsing::{Mutable, Type},
        Node,
    },
};

#[derive(Debug, Clone)]
pub struct FunctionArg {
    pub name: Ident,
    pub ty: Node<Type>,
    pub mutable: Mutable,
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub callee: Box<Node<Expr>>,
    pub args: Vec<Node<Expr>>,
}

#[derive(Debug, Clone)]
pub struct BlockExpr {
    pub stmts: Vec<Node<Stmt>>,
    pub tail: Option<Box<Node<Expr>>>,
}

#[derive(Debug, Clone)]
pub struct LoopExpr {
    pub block: Node<BlockExpr>,
}

#[derive(Debug, Clone)]
pub struct IfExpr {
    pub expr: Box<Node<Expr>>,
    pub block: Node<BlockExpr>,
    pub else_block: Option<Box<Node<Expr>>>,
}

#[derive(Debug, Clone)]
pub struct ClosureExpr {
    pub args: Vec<FunctionArg>,
    pub return_type: Node<Type>,
    pub block: Node<BlockExpr>,
}

#[derive(Debug, Clone)]
pub struct SwitchExpr {
    pub expr: Box<Node<Expr>>,
    pub cases: Vec<(Node<Expr>, Node<BlockExpr>)>,
    pub default: Option<Node<BlockExpr>>,
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
    Implicit,
    Range(Option<Box<Node<Expr>>>, Option<Box<Node<Expr>>>),
    MemberAccess(Box<Node<Expr>>, Ident),
    StructInit(Ident, Vec<(Ident, Node<Expr>)>),
    ArrayInit(Vec<Node<Expr>>),
    Index(Box<Node<Expr>>, Box<Node<Expr>>),
}

impl Expr {
    pub fn requires_semi(&self) -> bool {
        !matches!(
            self,
            Expr::If(_)
                | Expr::Loop(_)
                | Expr::Switch(_)
                | Expr::Block(_)
                | Expr::Closure(_)
                | Expr::Return(_)
                | Expr::Break(_)
                | Expr::Continue
        )
    }
}
