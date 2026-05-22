use crate::{
    aliases::Result,
    analyzer::{
        analyzer::{Analyzer, Fallback},
        symbol_passes::symbol_res::Environment,
        tables::struct_table::StructId,
    },
    attempt,
    err::{HarpyError, Kind},
    lexer::tokens::Lit,
    parser::{
        expr::expr_defs::Expr,
        types::type_parsing::{FunctionType, Type},
        Node,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub usize);

impl TypeId {
    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }

    pub fn get<'a>(&'a self, analyzer: &'a Analyzer) -> &'a ResolvedType {
        analyzer.db.type_table.get(*self)
    }

    pub fn void() -> Self {
        TypeId(5)
    }

    pub fn bool() -> Self {
        TypeId(3)
    }

    pub fn never() -> Self {
        TypeId(6)
    }

    pub fn unknown() -> Self {
        TypeId(0)
    }

    pub fn int() -> Self {
        TypeId(1)
    }

    pub fn str() -> Self {
        TypeId(4)
    }

    pub fn float() -> Self {
        TypeId(2)
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

    Ref(TypeId, bool),
    Boxed(TypeId, bool),
    Array(TypeId, Option<u64>),
    Range(TypeId),
    Struct(StructId),
    Function {
        args: Vec<TypeId>,
        return_type: TypeId,
    },
}

impl Analyzer {
    fn resolve_array_type(
        &mut self,
        ty: &Type,
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
                        return HarpyError::err(lit.span(), Kind::ArraySizeInt);
                    }
                }
                _ => return HarpyError::err(expr.span, Kind::ArraySizeInt.into()),
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
            .map(|arg| attempt!(self.resolve_type_with_env(arg, env)))
            .collect::<Vec<_>>();
        let return_type = attempt!(self.resolve_type_with_env(&fn_ty.return_type, env));
        let ty = ResolvedType::Function { args, return_type };
        Ok(self.db.type_table.register(ty))
    }

    fn resolve_inner_type(
        &mut self,
        parser_type: &Type,
        env: Option<&Environment>,
    ) -> Result<TypeId> {
        let ty = match parser_type {
            Type::Void => ResolvedType::Void,
            Type::Boxed(inner, mutable) => {
                let resolved = self.resolve_inner_type(inner, env)?;
                ResolvedType::Boxed(resolved, *mutable)
            }
            Type::Ref(inner, mutable) => {
                let resolved = self.resolve_inner_type(inner, env)?;
                ResolvedType::Ref(resolved, *mutable)
            }
            Type::FunctionType(f) => return self.resolve_fn_type(f, env),
            Type::Array(ty, s) => return self.resolve_array_type(ty, s, env),
            Type::Int => ResolvedType::Int,
            Type::Bool => ResolvedType::Bool,
            Type::Str => ResolvedType::Str,
            Type::Float => ResolvedType::Float,
            Type::Custom(name) => {
                let id = self.resolve_local_struct(env, name);
                if !id.is_valid() {
                    return HarpyError::err(name.span(), Kind::UnknownType(name.value().clone()));
                }
                ResolvedType::Struct(id)
            }
            Type::Unknown => ResolvedType::Unknown,
        };

        Ok(self.db.type_table.register(ty))
    }

    fn validate_type_structure(&mut self, ty: &Node<Type>) -> Result<()> {
        match &ty.inner {
            Type::Ref(inner_node, _) => {
                if let Type::Ref(_, _) = &inner_node.inner {
                    return HarpyError::err(ty.span, Kind::RecursiveRef);
                }
            }
            Type::Boxed(inner_node, _) => {
                if let Type::Boxed(_, _) = &inner_node.inner {
                    return HarpyError::err(ty.span, Kind::RecursiveBox);
                }
                if let Type::Ref(_, _) = &inner_node.inner {
                    return HarpyError::err(ty.span, Kind::BoxedRef);
                }
            }
            Type::Array(inner_node, _) => {
                self.validate_type_structure(inner_node)?;
            }
            Type::FunctionType(f) => {
                for arg in &f.args {
                    self.validate_type_structure(arg)?;
                }
                self.validate_type_structure(&f.return_type)?;
            }
            _ => {}
        }
        Ok(())
    }

    pub(in crate::analyzer) fn resolve_type_with_env(
        &mut self,
        ty: &Node<Type>,
        env: Option<&Environment>,
    ) -> Result<TypeId> {
        if let Some(id) = self.db.type_table.is_cached(ty.id) {
            return Ok(id);
        }
        self.validate_type_structure(ty)?;
        let resolved = self.resolve_inner_type(ty, env)?;
        self.db.type_table.cache(ty.id, resolved);
        Ok(resolved)
    }

    pub(in crate::analyzer) fn resolve_type(&mut self, ty: &Node<Type>) -> Result<TypeId> {
        self.resolve_type_with_env(ty, None)
    }
}
