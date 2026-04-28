use crate::{
    lexer::tokens::Ident,
    parser::{
        expr::expr_defs::{BlockExpr, Expr},
        types::type_parsing::Type,
        Node, Parser,
    },
};

pub struct ForStmt {
    pub iter_expr: Node<Expr>,
    pub temp_var: Node<Ident>,
    pub block: Node<BlockExpr>,
}

pub struct WhileStmt {
    pub expr: Node<Expr>,
    pub block: Node<BlockExpr>,
}

pub struct LetStmt {
    pub name: Node<Ident>,
    pub ttype: Option<Node<Type>>,
    pub expr: Option<Node<Expr>>,
}

pub struct FunctionDecl {
    pub name: Node<Ident>,
    pub args: Vec<(Node<Ident>, Node<Type>)>,
    pub return_type: Node<Type>,
    pub block: Node<BlockExpr>,
}

pub struct GlobalStmt {
    pub name: Node<Ident>,
    pub ttype: Node<Type>,
    pub expr: Node<Expr>,
}

impl<'parser> Parser<'parser> {}
