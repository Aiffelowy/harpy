use crate::{
    aliases::Result,
    lexer::tokens::Ident,
    parse_separated,
    parser::{
        expr::expr_defs::{BlockExpr, Expr, FunctionArg},
        types::type_parsing::Type,
        Node, Parser,
    },
    peek_and_consume, t, tt,
};

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub iter_expr: Node<Expr>,
    pub temp_var: Ident,
    pub block: Node<BlockExpr>,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub expr: Node<Expr>,
    pub block: Node<BlockExpr>,
}

#[derive(Debug, Clone)]
pub struct LetStmt {
    pub name: Ident,
    pub mutable: bool,
    pub ttype: Option<Node<Type>>,
    pub expr: Option<Node<Expr>>,
}

#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: Ident,
    pub args: Vec<Node<FunctionArg>>,
    pub return_type: Node<Type>,
    pub block: Node<BlockExpr>,
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
pub struct Program {
    pub items: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    For(Node<ForStmt>),
    While(Node<WhileStmt>),
    Let(Node<LetStmt>),
    Function(Node<FunctionDecl>),
    Global(Node<GlobalStmt>),
    Struct(Node<StructDecl>),
    Semi(Node<Expr>),
    Expr(Node<Expr>),
}

impl<'parser> Parser<'parser> {
    fn parse_let_stmt(&mut self) -> Result<LetStmt> {
        self.consume::<t!(let)>()?;
        let mutable = peek_and_consume!(self, mut);
        let name = self.consume()?;
        let mut ttype = None;
        let mut expr = None;
        if peek_and_consume!(self, :) {
            ttype = Some(self.parse_node(Self::parse_type_with_infer)?);
        }

        if peek_and_consume!(self, =) {
            expr = Some(self.parse_node(Self::parse_expr)?);
        }

        self.consume::<t!(;)>()?;

        Ok(LetStmt {
            name,
            ttype,
            expr,
            mutable,
        })
    }

    fn parse_while_stmt(&mut self) -> Result<WhileStmt> {
        self.consume::<t!(while)>()?;

        let expr = self.parse_node(Self::parse_expr)?;
        let block = self.parse_node(Self::parse_block_expr)?;

        Ok(WhileStmt { expr, block })
    }

    fn parse_for_stmt(&mut self) -> Result<ForStmt> {
        self.consume::<t!(for)>()?;
        let name = self.consume()?;
        self.consume::<t!(in)>()?;
        let expr = self.parse_node(Self::parse_expr)?;
        let block = self.parse_node(Self::parse_block_expr)?;

        Ok(ForStmt {
            iter_expr: expr,
            temp_var: name,
            block,
        })
    }

    fn parse_fn_decl(&mut self) -> Result<FunctionDecl> {
        self.consume::<t!(fn)>()?;
        let name = self.consume()?;
        let args = parse_separated!(self, "(", ")",,, self.parse_node(Self::parse_function_arg)?);
        let return_type = self.parse_node(Self::parse_function_return_type)?;
        let block = self.parse_node(Self::parse_block_expr)?;

        Ok(FunctionDecl {
            name,
            args,
            return_type,
            block,
        })
    }

    fn parse_global_stmt(&mut self) -> Result<GlobalStmt> {
        self.consume::<t!(global)>()?;
        let mutable = peek_and_consume!(self, mut);
        let name = self.consume()?;
        self.consume::<t!(:)>()?;
        let ttype = self.parse_node(Self::parse_type_with_infer)?;
        self.consume::<t!(=)>()?;
        let expr = self.parse_node(Self::parse_expr)?;
        self.consume::<t!(;)>()?;

        Ok(GlobalStmt {
            name,
            ttype,
            expr,
            mutable,
        })
    }

    fn parse_struct_field(&mut self) -> Result<StructField> {
        let name = self.consume()?;
        self.consume::<t!(:)>()?;
        let ttype = self.parse_node(Self::parse_type)?;
        Ok(StructField { name, ttype })
    }

    fn parse_struct_decl(&mut self) -> Result<StructDecl> {
        self.consume::<t!(struct)>()?;
        let name = self.consume()?;
        let fields = parse_separated!(self, "{", "}",;, self.parse_node(Self::parse_struct_field)?);
        Ok(StructDecl { name, fields })
    }

    pub(in crate::parser) fn parse_stmt(&mut self) -> Result<Stmt> {
        match self.peek()? {
            tt!(let) => Ok(Stmt::Let(self.parse_node(Self::parse_let_stmt)?)),
            tt!(while) => Ok(Stmt::While(self.parse_node(Self::parse_while_stmt)?)),
            tt!(for) => Ok(Stmt::For(self.parse_node(Self::parse_for_stmt)?)),
            tt!(fn) => {
                let decl = self.parse_node(Self::parse_fn_decl)?;
                peek_and_consume!(self, ;);
                Ok(Stmt::Function(decl))
            }
            tt!(struct) => {
                let decl = self.parse_node(Self::parse_struct_decl)?;
                peek_and_consume!(self, ;);
                Ok(Stmt::Struct(decl))
            }
            tt!(global) => Ok(Stmt::Global(self.parse_node(Self::parse_global_stmt)?)),

            _ => {
                let expr = self.parse_node(Self::parse_expr)?;
                if let tt!("}") = self.peek()? {
                    return Ok(Stmt::Expr(expr));
                }

                if expr.inner.requires_semi() {
                    self.consume::<t!(;)>()?;
                } else {
                    peek_and_consume!(self, ;);
                }
                Ok(Stmt::Semi(expr))
            }
        }
    }

    pub(in crate::parser) fn parse_program(&mut self) -> Result<Program> {
        let mut items = Vec::new();
        loop {
            if let tt!(eof) = self.peek()? {
                break;
            }

            let item_result = match self.peek()? {
                tt!(fn) => {
                    let decl = self.parse_node(Self::parse_fn_decl)?;
                    Ok(Stmt::Function(decl))
                }
                tt!(global) => {
                    let decl = self.parse_node(Self::parse_global_stmt)?;
                    Ok(Stmt::Global(decl))
                }
                tt!(struct) => {
                    let decl = self.parse_node(Self::parse_struct_decl)?;
                    Ok(Stmt::Struct(decl))
                }
                _ => self.unexpected("top-level item (fn, global, struct)"),
            };

            match item_result {
                Ok(stmt) => items.push(stmt),
                Err(e) => self.report_error(e, &[tt!(fn), tt!(global), tt!(struct)])?,
            }
        }

        Ok(Program { items })
    }
}
