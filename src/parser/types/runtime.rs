use crate::semantic_analyzer::type_table::RuntimeTypeIndex;

use super::BaseType;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum RuntimeType {
    Base(BaseType),
    Boxed(RuntimeTypeIndex),
    Ref(RuntimeTypeIndex),
    Void,
}
