use crate::{
    aliases::Result,
    analyzer::{
        analyzer::{Analyzer, Fallback},
        err::SymbolDeclError,
        symbol_res::Environment,
        tables::struct_table::StructId,
    },
    err::HarpyError,
    lexer::tokens::Lit,
    parser::{
        expr::expr_defs::Expr,
        types::type_parsing::{BaseType, FunctionType, Mutable, Type, TypeInner},
        Node,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub usize);

impl TypeId {
    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }
}
impl Fallback<TypeId> for Analyzer {
    fn fallback(&self) -> TypeId {
        TypeId(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResolvedType {
    Int,
    Float,
    Bool,
    Str,
    Void,
    Never,
    Unknown,

    Ref(TypeId, Mutable),
    Boxed(TypeId),
    Array(TypeId, Option<u64>),
    Struct(StructId),
    Function {
        args: Vec<TypeId>,
        return_type: TypeId,
    },
}

impl Analyzer {
    fn resolve_type_base(
        &mut self,
        base_type: &Node<BaseType>,
        env: Option<&Environment>,
    ) -> Result<TypeId> {
        let bt = match &base_type.inner {
            BaseType::Int => ResolvedType::Int,
            BaseType::Bool => ResolvedType::Bool,
            BaseType::Str => ResolvedType::Str,
            BaseType::Float => ResolvedType::Float,
            BaseType::Custom(name) => {
                if let Some(id) = env.and_then(|e| e.resolve_local_type(name.value())) {
                    return Ok(id);
                }
                if let Some(struct_id) = self.current_module().structs.get(name.value()) {
                    ResolvedType::Struct(*struct_id)
                } else {
                    return HarpyError::analyzer(
                        SymbolDeclError::UnknownType(name.value().clone()).into(),
                        name.span(),
                    );
                }
            }
        };

        Ok(self.db.type_table.register(bt))
    }

    fn resolve_array_type(
        &mut self,
        ty: &TypeInner,
        expr: &Option<Box<Node<Expr>>>,
        env: Option<&Environment>,
    ) -> Result<TypeId> {
        let resolved = self.resolve_inner_type(ty, env)?;
        let mut size = None;
        if let Some(expr) = expr {
            match &expr.inner {
                Expr::Literal(lit) => {
                    if let Lit::LitInt(i) = lit.value() {
                        size = Some(*i);
                    } else {
                        return HarpyError::analyzer(
                            SymbolDeclError::ArraySizeInt.into(),
                            lit.span(),
                        );
                    }
                }
                _ => return HarpyError::analyzer(SymbolDeclError::ArraySizeInt.into(), expr.span),
            }
        }
        let ty = ResolvedType::Array(resolved, size);
        Ok(self.db.type_table.register(ty))
    }

    pub(in crate::analyzer) fn resolve_fn_type(
        &mut self,
        fn_ty: &FunctionType,
        env: Option<&Environment>,
    ) -> Result<TypeId> {
        let args = fn_ty
            .args
            .iter()
            .map(|arg| self.resolve_inner_type(&arg.inner.inner, env))
            .collect::<Result<Vec<_>>>()?;
        let return_type = self.resolve_inner_type(&fn_ty.return_type.inner.inner, env)?;
        let ty = ResolvedType::Function { args, return_type };
        Ok(self.db.type_table.register(ty))
    }

    fn resolve_inner_type(
        &mut self,
        parser_type: &TypeInner,
        env: Option<&Environment>,
    ) -> Result<TypeId> {
        let ty = match parser_type {
            TypeInner::Base(base_node) => return self.resolve_type_base(base_node, env),
            TypeInner::Void => ResolvedType::Void,
            TypeInner::Boxed(inner) => {
                let resolved = self.resolve_inner_type(inner, env)?;
                ResolvedType::Boxed(resolved)
            }

            TypeInner::FunctionType(f) => return self.resolve_fn_type(f, env),
            TypeInner::Array(ty, s) => return self.resolve_array_type(ty, s, env),
            TypeInner::Unknown => ResolvedType::Unknown,
        };

        Ok(self.db.type_table.register(ty))
    }

    pub(in crate::analyzer) fn resolve_type_with_env(
        &mut self,
        ty: &Node<Type>,
        env: Option<&Environment>,
    ) -> Result<TypeId> {
        if let Some(id) = self.db.type_table.is_cached(ty.id) {
            return Ok(id);
        }

        let ty_inner = &ty.inner.inner;
        let resolved = self.resolve_inner_type(ty_inner, env)?;
        if ty.is_ref.0 {
            let resolved = ResolvedType::Ref(resolved, ty.is_ref.1);
            let ty_id = self.db.type_table.register(resolved);
            self.db.type_table.cache(ty.id, ty_id);
            return Ok(ty_id);
        }

        self.db.type_table.cache(ty.id, resolved);
        Ok(resolved)
    }

    pub(in crate::analyzer) fn resolve_type(&mut self, ty: &Node<Type>) -> Result<TypeId> {
        self.resolve_type_with_env(ty, None)
    }
}
