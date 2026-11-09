use crate::{
    lexer::tokens::{Ident, Literal},
    parser::{expr::ops::*, stmt::stmts::RichStmt, types::type_parsing::Type, Node},
};

#[derive(Debug, Clone)]
pub struct FunctionArg {
    pub name: Ident,
    pub ty: Node<Type>,
    pub mutable: bool,
}

#[derive(Debug, Clone)]
pub struct RichCallExpr {
    pub callee: Node<RichExpr>,
    pub args: Vec<Node<RichExpr>>,
}

#[derive(Debug, Clone)]
pub struct RichBlockExpr {
    pub stmts: Vec<Node<RichStmt>>,
    pub tail: Option<Node<RichExpr>>,
}

#[derive(Debug, Clone)]
pub struct RichLoopExpr {
    pub block: Node<RichBlockExpr>,
}

#[derive(Debug, Clone)]
pub struct RichIfExpr {
    pub expr: Node<RichExpr>,
    pub block: Node<RichBlockExpr>,
    pub else_block: Option<Node<RichExpr>>,
}

#[derive(Debug, Clone)]
pub struct RichClosureExpr {
    pub args: Vec<FunctionArg>,
    pub return_type: Node<Type>,
    pub block: Node<RichBlockExpr>,
}

#[derive(Debug, Clone)]
pub struct SwitchExpr {
    pub expr: Node<RichExpr>,
    pub cases: Vec<(Node<RichExpr>, Node<RichBlockExpr>)>,
    pub default: Option<Node<RichBlockExpr>>,
}

#[derive(Debug, Clone)]
pub struct RichBreakExpr {
    pub expr: Option<Node<RichExpr>>,
}

#[derive(Debug, Clone)]
pub struct RichReturnExpr {
    pub expr: Option<Node<RichExpr>>,
}

#[derive(Debug, Clone)]
pub enum RichExpr {
    Infix(Node<RichExpr>, InfixOp, Node<RichExpr>),
    Prefix(PrefixOp, Node<RichExpr>),
    Assign(Node<RichExpr>, AssignOp, Node<RichExpr>),
    Literal(Literal),
    Ident(Ident),
    Call(Node<RichCallExpr>),
    Loop(Node<RichLoopExpr>),
    Return(Option<Node<RichExpr>>),
    Break(Option<Node<RichExpr>>),
    Continue,
    If(Node<RichIfExpr>),
    Switch(Node<SwitchExpr>),
    Block(Node<RichBlockExpr>),
    Closure(Node<RichClosureExpr>),
    Borrow(Node<RichExpr>, bool),
    Box(Node<RichExpr>, bool),
    Implicit,
    Range(Option<Node<RichExpr>>, Option<Node<RichExpr>>),
    MemberAccess(Node<RichExpr>, Ident),
    StructInit(Ident, Vec<(Ident, Node<RichExpr>)>),
    ArrayInit(Vec<Node<RichExpr>>),
    Index(Node<RichExpr>, Node<RichExpr>),
    Clone(Node<RichExpr>),
}

impl RichExpr {
    pub fn requires_semi(&self) -> bool {
        !matches!(
            self,
            RichExpr::If(_)
                | RichExpr::Loop(_)
                | RichExpr::Switch(_)
                | RichExpr::Block(_)
                | RichExpr::Closure(_)
                | RichExpr::Return(_)
                | RichExpr::Break(_)
                | RichExpr::Continue
        )
    }
}
