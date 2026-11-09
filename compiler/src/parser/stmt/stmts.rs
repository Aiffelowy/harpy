use std::convert::Infallible;

use crate::{
    aliases::Result,
    parse_separated,
    parser::{
        expr::{expr_defs::RichExpr, simple_expr_defs::Expr},
        stmt::stmt_structs::{
            ForStmt, LetStmt, RichFunctionDecl, RichGlobalStmt, RichLetStmt, RichProgram,
            StructDecl, StructField, WhileStmt,
        },
        Node, Parser,
    },
    peek_and_consume, t, tt,
};

#[derive(Debug, Clone)]
pub enum RichStmt {
    For(Node<ForStmt>),
    While(Node<WhileStmt>),
    Let(Node<RichLetStmt>),
    Semi(Node<RichExpr>),
    Expr(Node<RichExpr>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(Node<LetStmt>),
    Semi(Node<Expr>),
    Expr(Node<Expr>),
}

impl<'parser> Parser<'parser> {
    fn parse_let_stmt(&mut self) -> Result<RichLetStmt> {
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

        Ok(RichLetStmt {
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
        let start = self.parse_node(Self::parse_expr)?;
        self.consume::<t!(=>)>()?;
        let end = self.parse_node(Self::parse_expr)?;
        let block = self.parse_node(Self::parse_block_expr)?;

        Ok(ForStmt {
            start,
            end,
            temp_var: name,
            block,
        })
    }

    fn parse_fn_decl(&mut self) -> Result<RichFunctionDecl> {
        self.consume::<t!(fn)>()?;
        let name = self.consume()?;
        let args = parse_separated!(self, "(", ")",,, self.parse_node(Self::parse_function_arg)?);
        let return_type = self.parse_node(Self::parse_function_return_type)?;
        let block = self.parse_node(Self::parse_block_expr)?;

        Ok(RichFunctionDecl {
            name,
            args,
            return_type,
            block,
        })
    }

    fn parse_global_stmt(&mut self) -> Result<RichGlobalStmt> {
        self.consume::<t!(global)>()?;
        let mutable = peek_and_consume!(self, mut);
        let name = self.consume()?;
        self.consume::<t!(:)>()?;
        let ttype = self.parse_node(Self::parse_type_with_infer)?;
        self.consume::<t!(=)>()?;
        let expr = self.parse_node(Self::parse_expr)?;
        self.consume::<t!(;)>()?;

        Ok(RichGlobalStmt {
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

    pub(in crate::parser) fn parse_stmt(&mut self) -> Result<RichStmt> {
        let t = self.peek()?;
        match t {
            tt!(let) => Ok(RichStmt::Let(self.parse_node(Self::parse_let_stmt)?)),
            tt!(while) => Ok(RichStmt::While(self.parse_node(Self::parse_while_stmt)?)),
            tt!(for) => Ok(RichStmt::For(self.parse_node(Self::parse_for_stmt)?)),

            tt!(struct) | tt!(global) => self.unexpected("Nested items not allowed"),

            _ => {
                let expr = self.parse_node(Self::parse_expr)?;
                if let tt!("}") = self.peek()? {
                    return Ok(RichStmt::Expr(expr));
                }

                if expr.inner().requires_semi() {
                    self.consume::<t!(;)>()?;
                } else {
                    peek_and_consume!(self, ;);
                }
                Ok(RichStmt::Semi(expr))
            }
        }
    }

    pub(in crate::parser) fn parse_program(&mut self) -> Result<RichProgram> {
        let mut structs = Vec::new();
        let mut globals = Vec::new();
        let mut functions = Vec::new();

        loop {
            if let tt!(eof) = self.peek()? {
                break;
            }

            match self.peek()? {
                tt!(fn) => {
                    let decl = self.parse_node(Self::parse_fn_decl)?;
                    functions.push(decl);
                }
                tt!(global) => {
                    let decl = self.parse_node(Self::parse_global_stmt)?;
                    globals.push(decl);
                }
                tt!(struct) => {
                    let decl = self.parse_node(Self::parse_struct_decl)?;
                    structs.push(decl);
                }
                _ => {
                    let err = self
                        .unexpected::<Infallible>("top-level item (fn, global, struct)")
                        .err()
                        .unwrap();
                    self.report_error(err, &[tt!(fn), tt!(global), tt!(struct)])?
                }
            };
        }

        Ok(RichProgram {
            structs,
            globals,
            functions,
        })
    }
}
