use crate::{
    aliases::Result,
    err::HarpyError,
    extensions::SymbolInfoRefExt,
    lexer::tokens::{Ident, Lit, Literal},
    parser::{
        expr::{
            expr::{CallExpr, SpannedExpr},
            infix::InfixOp,
            prefix::PrefixOp,
            Expr,
        },
        node::Node,
        types::{BaseType, PrimitiveType, Type, TypeInner},
    },
    semantic_analyzer::{
        analyzer::Analyzer,
        err::SemanticError,
        symbol_info::{BorrowInfo, SymbolInfoKind},
    },
};

use super::{infix_resolver::InfixResolver, prefix_resolver::PrefixResolver};

#[derive(Debug, Clone, Copy)]
pub enum ResolveMode {
    Read,
    Write,
}

pub struct ExprResolver;

impl ExprResolver {
    fn resolve_lit(lit: &Node<Literal>, analyzer: &mut Analyzer) -> Type {
        let inner = match lit.value() {
            Lit::LitInt(_) => BaseType::Primitive(PrimitiveType::Int),
            Lit::LitFloat(_) => BaseType::Primitive(PrimitiveType::Float),
            Lit::LitStr(_) => BaseType::Primitive(PrimitiveType::Str),
            Lit::LitBool(_) => BaseType::Primitive(PrimitiveType::Bool),
            Lit::LitVoid => return Type::void(),
        };

        let t = Type {
            mutable: false,
            inner: TypeInner::Base(inner),
        };

        analyzer.register_constant(lit, &t);
        t
    }

    fn resolve_ident(
        ident: &Node<Ident>,
        analyzer: &mut Analyzer,
        mode: ResolveMode,
    ) -> Result<Type> {
        let sym_ref = analyzer.get_symbol(&**ident)?;
        let symbol = (*sym_ref).borrow();
        if let SymbolInfoKind::Variable(ref v) = symbol.kind {
            match mode {
                ResolveMode::Read => {
                    if !v.initialized {
                        return HarpyError::semantic(SemanticError::UninitializedVar, ident.span());
                    }
                }
                ResolveMode::Write => (),
            }
        }

        // Map this identifier to its local address if it's a variable or parameter
        analyzer.map_ident_to_local_with_symbol(ident, &symbol);

        Ok(symbol.ty.ttype.clone())
    }

    fn resolve_call(
        expr: &Node<CallExpr>,
        analyzer: &mut Analyzer,
        mode: ResolveMode,
    ) -> Result<Type> {
        let ident = &expr.ident;
        let params = &expr.args;

        let sym_ref = analyzer.get_symbol(ident)?;
        let symbol = (*sym_ref).borrow();

        let func_info = match &symbol.kind {
            SymbolInfoKind::Function(f) => f,
            _ => return HarpyError::semantic(SemanticError::NotAFunc(ident.clone()), ident.span()),
        };

        if params.len() != func_info.params.len() {
            return HarpyError::semantic(
                SemanticError::ArgCountMismatch(
                    ident.clone(),
                    params.len(),
                    func_info.params.len(),
                ),
                ident.span(),
            );
        }

        for (param_expr, param_type) in params.iter().zip(&func_info.params) {
            let ttype = Self::resolve_expr(param_expr, analyzer, mode)?;
            let param_t = &param_type.ttype;
            if !param_t.param_compatible(&ttype) {
                return HarpyError::semantic(
                    SemanticError::ArgTypeMismatch(ttype, param_type.clone()),
                    param_expr.span(),
                );
            }
        }

        analyzer.register_call(ident, expr);

        Ok(symbol.ty.ttype.clone())
    }

    fn resolve_prefix(
        op: &PrefixOp,
        rhs: &Expr,
        analyzer: &mut Analyzer,
        mode: ResolveMode,
    ) -> Result<Type> {
        let rhs_type = Self::resolve_expr(rhs, analyzer, mode)?;
        PrefixResolver::resolve(op, &rhs_type)
    }

    fn resolve_borrow(expr: &SpannedExpr, mutable: bool, analyzer: &mut Analyzer) -> Result<Type> {
        let Expr::Ident(ref i) = **expr else {
            return HarpyError::semantic(SemanticError::InvalidBorrow, expr.span());
        };
        let symbol = analyzer.get_symbol(i)?;
        let ttype = symbol.get().ty.clone();

        if mutable && !ttype.mutable {
            return HarpyError::semantic(SemanticError::BorrowMutNonMutable, expr.span());
        }
        {
            let mut var = symbol.as_variable_mut().ok_or_else(|| {
                HarpyError::new(
                    crate::err::HarpyErrorKind::SemanticError(SemanticError::InvalidVarBorrow(
                        symbol.get().kind.clone(),
                    )),
                    expr.span(),
                )
            })?;

            if !var.initialized {
                return HarpyError::semantic(SemanticError::UninitializedVar, i.span());
            }

            if mutable && var.immutably_borrowed_count != 0 {
                return HarpyError::semantic(
                    SemanticError::CreatedMutableBorrowWhileImmutableBorrow,
                    expr.span(),
                );
            }

            if var.mutably_borrowed {
                return HarpyError::semantic(SemanticError::AlreadyMutablyBorrowed, expr.span());
            }

            if mutable {
                var.mutably_borrowed = true;
            } else {
                var.immutably_borrowed_count += 1;
            }

            analyzer.register_borrow(BorrowInfo {
                depth: analyzer.current_depth(),
                original: symbol.clone(),
                borrow_span: expr.span(),
            });
        }
        // Map the borrowed identifier to its local address for code generation
        analyzer.map_ident_to_local_with_symbol(i, &symbol.get());

        Ok(Type {
            mutable,
            inner: TypeInner::Ref(Box::new(Type {
                mutable,
                inner: ttype.inner.clone(),
            })),
        })
    }

    fn resolve_infix(
        lhs: &Expr,
        op: &InfixOp,
        rhs: &Expr,
        analyzer: &mut Analyzer,
        mode: ResolveMode,
    ) -> Result<Type> {
        let lhs_type = Self::resolve_expr(lhs, analyzer, mode)?;
        let rhs_type = Self::resolve_expr(rhs, analyzer, mode)?;
        InfixResolver::resolve(op, &lhs_type, &rhs_type)
    }

    pub fn resolve_expr(expr: &Expr, analyzer: &mut Analyzer, mode: ResolveMode) -> Result<Type> {
        match expr {
            Expr::Literal(l) => Ok(Self::resolve_lit(l, analyzer)),
            Expr::Ident(i) => Self::resolve_ident(i, analyzer, mode),
            Expr::Call(expr) => Self::resolve_call(expr, analyzer, mode),
            Expr::Prefix(op, rhs) => Self::resolve_prefix(op, rhs, analyzer, mode),
            Expr::Infix(lhs, op, rhs) => Self::resolve_infix(lhs, op, rhs, analyzer, mode),
            Expr::Borrow(expr, mutable) => Self::resolve_borrow(expr, *mutable, analyzer),
        }
    }
}
