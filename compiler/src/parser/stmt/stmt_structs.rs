use crate::{
    lexer::tokens::Ident,
    parser::{
        expr::{
            expr_defs::{FunctionArg, RichBlockExpr, RichExpr},
            simple_expr_defs::{BlockExpr, Expr},
        },
        types::type_parsing::Type,
        Node,
    },
};

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub start: Node<RichExpr>,
    pub end: Node<RichExpr>,
    pub temp_var: Ident,
    pub block: Node<RichBlockExpr>,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub expr: Node<RichExpr>,
    pub block: Node<RichBlockExpr>,
}

#[derive(Debug, Clone)]
pub struct RichLetStmt {
    pub name: Ident,
    pub mutable: bool,
    pub ttype: Option<Node<Type>>,
    pub expr: Option<Node<RichExpr>>,
}

#[derive(Debug, Clone)]
pub struct LetStmt {
    pub name: Ident,
    pub mutable: bool,
    pub ttype: Option<Node<Type>>,
    pub expr: Option<Node<Expr>>,
}

#[derive(Debug, Clone)]
pub struct RichFunctionDecl {
    pub name: Ident,
    pub args: Vec<Node<FunctionArg>>,
    pub return_type: Node<Type>,
    pub block: Node<RichBlockExpr>,
}

#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: Ident,
    pub args: Vec<Node<FunctionArg>>,
    pub return_type: Node<Type>,
    pub block: Node<BlockExpr>,
}

#[derive(Debug, Clone)]
pub struct RichGlobalStmt {
    pub name: Ident,
    pub mutable: bool,
    pub ttype: Node<Type>,
    pub expr: Node<RichExpr>,
}

#[derive(Debug, Clone)]
pub struct GlobalStmt {
    pub name: Ident,
    pub mutable: bool,
    pub ttype: Node<Type>,
    pub expr: Node<Expr>,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: Ident,
    pub ttype: Node<Type>,
}

#[derive(Debug, Clone)]
pub struct StructDecl {
    pub name: Ident,
    pub fields: Vec<Node<StructField>>,
}

#[derive(Debug, Clone)]
pub struct RichProgram {
    pub structs: Vec<Node<StructDecl>>,
    pub functions: Vec<Node<RichFunctionDecl>>,
    pub globals: Vec<Node<RichGlobalStmt>>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub structs: Vec<Node<StructDecl>>,
    pub functions: Vec<Node<FunctionDecl>>,
    pub globals: Vec<Node<GlobalStmt>>,
}
