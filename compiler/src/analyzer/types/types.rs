use crate::analyzer::tables::struct_table::StructId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResolvedType {
    Int,
    Float,
    Bool,
    Str,
    Void,
    Never,
    Unknown,

    Ref(TypeId),
    Boxed(TypeId),
    Array(TypeId, Option<u64>),
    Struct(StructId),
    Function {
        args: Vec<TypeId>,
        return_type: TypeId,
    },
}
