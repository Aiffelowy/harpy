use crate::{
    aliases::Result,
    err::HarpyError,
    extensions::SymbolInfoRefExt,
    lexer::tokens::{Ident, Lit, Literal},
    parser::{
        expr::{expr::SpannedExpr, infix::InfixOp, prefix::PrefixOp, Expr},
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

pub struct ExprResolver;

impl ExprResolver {
    fn resolve_lit(lit: &Node<Literal>, analyzer: &mut Analyzer) -> Type {
        let inner = match lit.value() {
            Lit::LitInt(_) => BaseType::Primitive(PrimitiveType::Int),
            Lit::LitFloat(_) => BaseType::Primitive(PrimitiveType::Float),
            Lit::LitStr(_) => BaseType::Primitive(PrimitiveType::Str),
            Lit::LitBool(_) => BaseType::Primitive(PrimitiveType::Bool),
        };

        let t = Type {
            mutable: false,
            inner: TypeInner::Base(inner),
        };

        analyzer.register_constant(&lit, &t);
        t
    }

    fn resolve_ident(ident: &Ident, analyzer: &mut Analyzer) -> Result<Type> {
        let sym_ref = analyzer.get_symbol(ident)?;
        let symbol = (*sym_ref).borrow();
        Ok(symbol.ty.ttype.clone())
    }

    fn resolve_call(ident: &Ident, params: &[Node<Expr>], analyzer: &mut Analyzer) -> Result<Type> {
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
            let ttype = Self::resolve_expr(param_expr, analyzer)?;
            let param_t = &param_type.ttype;
            if !param_t.param_compatible(&ttype) {
                //fix
                return HarpyError::semantic(
                    SemanticError::ArgTypeMismatch(ttype, param_type.clone()),
                    param_expr.span(),
                );
            }
        }

        Ok(symbol.ty.ttype.clone())
    }

    fn resolve_prefix(op: &PrefixOp, rhs: &Expr, analyzer: &mut Analyzer) -> Result<Type> {
        let rhs_type = Self::resolve_expr(rhs, analyzer)?;
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

        let mut var = symbol.as_variable_mut().ok_or_else(|| {
            HarpyError::new(
                crate::err::HarpyErrorKind::SemanticError(SemanticError::InvalidVarBorrow(
                    symbol.get().kind.clone(),
                )),
                expr.span(),
            )
        })?;

        println!("{:?}", var);

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
    ) -> Result<Type> {
        let lhs_type = Self::resolve_expr(lhs, analyzer)?;
        let rhs_type = Self::resolve_expr(rhs, analyzer)?;
        InfixResolver::resolve(op, &lhs_type, &rhs_type)
    }

    pub fn resolve_expr(expr: &Expr, analyzer: &mut Analyzer) -> Result<Type> {
        match expr {
            Expr::Literal(l) => Ok(Self::resolve_lit(l, analyzer)),
            Expr::Ident(i) => Self::resolve_ident(i, analyzer),
            Expr::Call(ident, params) => Self::resolve_call(ident, params, analyzer),
            Expr::Prefix(op, rhs) => Self::resolve_prefix(op, rhs, analyzer),
            Expr::Infix(lhs, op, rhs) => Self::resolve_infix(lhs, op, rhs, analyzer),
            Expr::Borrow(expr, mutable) => Self::resolve_borrow(expr, *mutable, analyzer),
        }
    }
}
